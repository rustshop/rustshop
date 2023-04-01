# Bootstraping the shop

This document describes how to bootstrap an AWS account
using `rustshop` as a base infrastructure for your own "shop",
using `rustshop`.

There will be some (relatively short) amount of manual work,
and then the automation will take over.

## Manual prep-work

### Pick a `<shopname>`

Bootstrap scripts will derive bunch of names from it.
It should have a solid chance to be unique, otherwise you risk
into running into name conflicts for globally unique resource
names (like S3 Buckets).

In our case `<shopname>` is `rustshop`.

### Get a DNS domain (`<domain>`)

A shop like this will most probably require a DNS domain. Technically
a domain is not strictly necessary until setting up a k8s
cluster, but `rustshop` requires a base domain name configuration early
in the process.

You can use AWS to host your domain, or you can use an external
DNS hosting provider.

If you want to use AWS to host the DNS, you might want to create the
root account first (see below), and head to Route 53 to purchase it.

In our case the `<domain>` is rustshop.org.

### Setup email

When generating accounts rustshop will need a root user email
address.

By default it will use a `<email-user>+<account-name>@<email-host>` email
address generation system, so you can use one email address,
with different labels. Look up "gmail plus addressing" if you're
not familiar with this scheme. Other email providers often
support it as well.

Emails do not have to be in the `<domain>`, but can be.

Figure it out - it's beyond of the scope of this document.

In our case the base email will be `infra@rustshop.org`.

### Create your root account

Create (if you haven't already) the root account for your
organization (which will have other sub-accounts).

Name the AWS account `<shopname>-root`. Consider using`<email-user>+root@<email-domain>`
as the account email to be consistent.

**Set a strong password.**

You might want to watch https://learn.cantrill.io/courses/730712/lectures/24950112
for some relevant instruction.

**Add MFA for this account.** Seriously - always set MFA on all your accounts.
You do not want to pay a lot of money because your password leaked and now
your account is mining some crypto.

Create an initial "Cost Budget" and add an alert in it. If anything goes
wrong you want to know that you're paying more than expected.

Create `iamadmin` IAM admin user with `AdministratorAccess` policy. Make sure
to allow it the "Access key - Programmatic access" option -
we are going to need it soon. You might want to watch
https://learn.cantrill.io/courses/730712/lectures/24950119 for instructions.

If you selected "Access key - Programmatic access" option, you should be presented
with access keys details. Keep it around safe locally. It will be need soon.

Though it isn't strictly necessary, if you are like me and AWS is a bit new
to you consider enrolling into
[Adrian Cantrill's AWS Certified Solutions Architect - Associate course](https://learn.cantrill.io/p/aws-certified-solutions-architect-associate-saa-c02).
The helpful videos linked above are the freely accessible parts of the
much larger course, and I can highly recommend it.


### Create your shop's monorepo git repository

Make sure you have Nix installed and set up with flake support.
See [README.onboarding](README.onboarding) for help.

You can start with `git init .`, or create it on remotely
and clone locally.

Copy the [flake.nix](rustshop/templates/flake.nix) to the
root dir. Then run:

```sh
git add flake.nix
nix flake update
```

and you should finally be able to get the rustshop shell
working with:

```sh
nix develop
```

From now on `nix develop` will be your entry point to working
with your shop.

A `shop` command appear in your shell:

```
myshop$ shop
rustshop 
Rustshop binary

USAGE:
    shop <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    add          Manually add rustshop components to track (see `bootstrap` instead)
    bootstrap    Set up new amazon account/organization
    configure    Configure user settings
    get          Display certain values
    help         Print this message or the help of the given subcommand(s)
    switch       Switch current context (account, cluster, namespace)

Help and feedback: https://github.com/rustshop/rustshop/discussions/categories/help-general
```

## Bootstrap your infra

### Set up `aws` command profile

**Make sure the credentials here are from the IAM `iamadmin` user**,
and not from the root account root user!

`aws` command should be available in shell and you can configure a profile
using `env RUSTSHOP_NO_BIN_WRAP=true aws configure --profile <shopname>-root` like this:

TODO: The `env RUSTSHOP_NO_BIN_WRAP=true` is needed because shop config is not yet
configured which messes up with `aws` wrapper.

```
infra$ aws configure
AWS Access Key ID [None]: SOMEKEYIDYOUVEGOT
AWS Secret Access Key [None]: SomeSecretKey1ThatAmazonProduced
Default region name [None]: us-east-1
Default output format [None]:
```

The Key ID and Secrect Access Key should be provided to you
when you created `iamadmin` user.

You can use `aws configure list-profiles` to list all profiles.

### Bootstrap your account

Run:

```sh
shop bootstrap shop --domain <domain> --email <email> <shopname>
shop bootstrap account prod --email <email>
```

Example:

```sh
shop bootstrap shop --domain rustshop.org --email admin@rustshop.org rustshop
shop bootstrap account prod --email admin@rustshop.org

````

Follow the output, and in case of any issues
[try asking for help](https://github.com/rustshop/rustshop/discussions/categories/help-general).

You might verify in the AWS Console, under Organizations product that and organization was
created, with a `<shopname>-prod` sub-account.


### Bootstrap prod k8s cluster

Since everything is already installed, basic infrastructure bootstrapped and `rustshop`
wraps `kops` to automatically supply the account-specific values, bootstrapping
a fresh kuberentes cluster is very simple.

Switch to the `prod` account:

```
shop switch account prod
```

#### Bootstrap cluster using `shop bootstrap`

`rustshop` can automate the k8s cluster bootstrapping by:

* creating a DNS zone for you and prompting you to configure it
* creating the `kops` cluster configuration by calling `kops create cluster` with right arugments

However since it is a multi-step process that you probably want
to customize and understand, after this section we will describe
the manual procedure that `shop bootstrap cluster` is automating.

Call:

```
shop switch account prod
shop bootstrap cluster prod --minimal
```

read, the prompts, configure and verify your DNS setup.

Note that at the time of writting `--minimal` option does
not lower the etcd EBS size to `1` and doesn't set up spot
instance settings on the nodes.

When ready call:

```
shop bootstrap cluster prod --minimal --dns-ready
```

then use `kops edit cluster` and `kops edit ig`, etc to customize
to your desired settings and when ready, call:

```
kops update cluster --yes
```

#### Bootstrap cluster manually

### DNS

`kops` cluster requires a DNS zone that it can use. We are going to use
Route 53 to host it.

Pick a DNS name for your cluster. `prod.k8s.<domain>` is recommended.

To create the hosted zone for the cluster run:

```sh
ID=$(uuidgen) && \
aws route53 create-hosted-zone \
--name prod.k8s.<domain> \
--caller-reference $ID \
| jq .DelegationSet.NameServers
```

You can use `aws route53 list-hosted-zones` and `aws route53 get-hosted-zone --id <zone-id>`
to discover the hosted zones you've created.

The output of this command should give you a list of DNS names to add as a NS record
to your main domain. Depending on your DNS hosting provider, the details might
differ, but it comes down to creating 4 NS records for `prod.k8s` in your <domain> host
zone configuration.


Verify your DNS with:

```sh
ping prod.k8s.<domain>
dig prod.k8s.<domain>
```

Do not move forward until the DNS is ready.

Create cluster configuration:

```sh
kops create cluster \
  --cloud aws \
  --zones "us-east-1f" \
  --master-count 1  \
  --master-size t3a.small \
  --master-volume-size 8 \
  --node-count 1 \
  --node-size t3a.small \
  --node-volume-size 8 \
  --networking=calico --topology public
```

If you are a cheapskate like me, you can
change the EBS volumes to bare minimums with:

```
kops edit cluster
```
    
and add `volumeSize: 1` on each etcd instances (https://unix.stackexchange.com/a/598838/4389), along with
    
```
    manager:
      backupInterval: 12h0m0s
```
    
to avoid accumulating a lot of backup in the state s3 backup.

And use spot instances:

```
kops get ig
kops edit ig <masterig> # add `maxPrice: "0.01"
kops edit ig <nodesig> # add `maxPrice: "0.01"
```

After tweaking and verifing the settings you can deploy the cluster with:

```
kops update cluster --yes
```

This process can easily create up to 30 minutes, so be patient.

You might need to run:

```
kops export kubecfg --admin
```

to update the kubectl context in `~/.kube/config`

After the cluster is bootstrapped you should be
able to execute:

```
kubectl get nodes
```

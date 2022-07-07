## Bootstraping the shop

This document describes how to bootstrap an AWS account
using `rustshop` as a base infrastructure for your own "shop",
using `rustshop`.

There will be some (relatively short) amount of manual work,
and then the automation will take over.


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

You can start with `git init .`, or create it on remotely
and clone locally.

Copy the [flake.nix](rustshop/templates/flake.nix) to the
root dir. `git add flake.nix`

### Bootstrap your infra

This is the time where automation takes over.

Make sure to clone your repo and set up Nix as in [Onboarding document](../README.onboarding.md)

```
~$ git clone https://github.com/rustshop/rustshop <shopname>
~$ cd <shopname>/infra  # change dir to infra inside the cloned repo
infra$ nix develop          # get the shell with all the infra tools you might need
```

`nix develop` might require you to perform some initial configuration. Please read
the prompts.

### Set up `aws` command profile

`aws` command should be available in shell and you can configure a profile
using `aws configure --profile <shopname>-root` like this:

Make sure the credentials here are from the IAM `iamadmin` user,
and not from the root account root user!

`nix develop` should have set your `$AWS_PROFILE` already:

```
infra$ nix develop
Setting AWS_PROFILE=rustshop-root
```

so you can use just `aws configure`. Enter the access key information
for IAM Admin User you created.

```
infra$ aws configure
AWS Access Key ID [None]: SOMEKEYIDYOUVEGOT
AWS Secret Access Key [None]: SomeSecretKey1ThatAmazonProduced
Default region name [None]: us-east-1
Default output format [None]:
```

You can use `aws configure list-profiles` to list all profiles.

### Bootstrap your account using `aws-bootstrap` command

You can try `aws-bootstrap --help` to get a better understanding of usage
and what is going on.

Run:

```
aws-bootstrap --base <shopname>  --email infra@<domain>
```

Follow the output, and in case of any issues
[try asking for help](https://github.com/rustshop/rustshop/discussions/categories/help-general).

You might verify in the AWS Console, under Organizations product that and organization was
created, with two sub-accounts: `<shopname>-prod` and `<shopname>-dev`. Your email account
should also receive emails about it.

Edit `.env` file with the acount ids returned by the `aws-bootstrap`.


### Setup terraform

Change directory to `account/root` and use `terraform init`:

```
infra$ cd account/root
root$ terraform init
[...] # should work
root$ terraform plan
[...] # should work
root$ terraform apply
[...] # should work
```

Initialize Terraform in other accounts: `dev` & `prod`

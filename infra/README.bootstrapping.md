## Bootstraping the shop

There will be some (relatively short) amount of manual work,
and then the automation will take over.

### Register domain

If you want to use AWS to host the DNS, you might want to create an
account first (see below). This isn't necessary - you can start
with the domain registered elsewhere, which will allow you to use
existing email in this domain as your root account root email.

In our case the `<domain>` is rustshop.org.

### Setup email

You need to be able to receive emails at `infra+<labels>@<domain>`.
Figure it out - it's beyond of the scope of this document.
(TODO: Point to some choices and instructions)

In our case the emails are `infra+<labels>@rustshop.org`.

### Create your root account

Pick a `<shopname>`. This will be used in a cuple of places.
It should have a solid chance to be unique, otherwise you risk
into running into name conflicts for globally unique resource
names (like S3 Buckets).

In our case the `<shopname>` is `rustshop`.

Name the AWS account `<shopname>-root` and use `infra+root@<domain>`
as the contact email. Set a strong password.

You might want to watch https://learn.cantrill.io/courses/730712/lectures/24950112
for some relevant instruction.

Add MFA for this account. Seriously - always set MFA on all your accounts.
You do not want to pay a lot of money because your password leaked and now
your account is mining some crypto.

Create an initial "Cost Budget", and add an alert in it. If anything goes
wrong you want to know that you're paying more than expected.

Create `iamadmin` IAM admin user with `AdministratorAccess` policy. Make sure
to allow it "Access key - Programmatic access" -
we are going to need it soon. You might want to watch
https://learn.cantrill.io/courses/730712/lectures/24950119 for instructions.

If you selected "Access key - Programmatic access" option, you should be presented
with access keys details. Keep it around safe locally. It will be need soon.

Though it isn't strictly necessary, but if you are like me and AWS is a bit new
to you  consider enrolling into
[Adrian Cantrill's AWS Certified Solutions Architect - Associate course](https://learn.cantrill.io/p/aws-certified-solutions-architect-associate-saa-c02).
The helpful videos linked above are the freely accessible parts of the
much larger course, and I can highly recommend it.

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

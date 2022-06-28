## Bootstraping the shop

There will be some (relatively short) amount of manual work,
and then the automation will take over.

### Register domain

If you want to use AWS to host the DNS, you might want to create an
account first (see below). This isn't necessary - you can start
with the domain registered elsewhere, which will allow you to use
existing email in this domain as your root account root email.

### Setup email

You need to be able to receive emails at `infra+<labels>@<domain>`

### Create your root account

Name it `<shopname>-root` and use `infra+root@<domain>` as the
contact email. Set a good random password.


Add MFA for this account. Seriously. You do not want to pay a lot
of money because your password leaked and now your account is mining
some crypto.

Create an initial "Cost Budget", and add an alert in it. If anything goes
wrong you want to know that you're paying more than expected. 

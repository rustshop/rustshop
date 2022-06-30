# AWS Bootstrap

Bootstrap a minimal self-hosted Terraform setup on an AWS account, using
CloudFormation stack.

Part of the [rustshop](https://github.com/rustshop/) project.


## Explanation

When called like this:

```
aws-bootstrap --email infra@example.com --base example --accounts dev,prod
```

with `AWS_PROFILE` pointing to an `iamadmin` Account Admin IAM identity on,
will:

* create a AWS Organinzation (if needed):
  * create two (sub-)accounts:
    * `example-dev` (email: infra+dev@example.com)
    * `example-prod` (email: infra+prod@example.com)
* in each account deploy a minimal CloudFormation stack including:
  * S3 Bucket for Terraform State
  * S3 Bucket for CloudWatch Logs
  * Minimal (thus drit cheap) DynamoDB for Terraform State locking
  * [some minimal policies, etc.](./src/cf-bootstrap.yaml)


## Requirements:

* Uses `aws` CLI command under the hood

# Building

You can use `./aws-bootstrap.build.sh` to build the `Dockerfile` an
export the statically linked Linux binary.

This is a normal Rust project, so can be build using cargo.

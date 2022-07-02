provider "aws" {
  region = local.aws_region
  assume_role {
      role_arn = "${local.aws_role_arn}"
  }
}

terraform {
  backend "s3" {
    # https://github.com/hashicorp/terraform/issues/13022#issuecomment-294262392
    # taken care of by `terraform-wrapper`
  }
}

resource "aws_s3_bucket" "images" {
  bucket = "${local.account_name}-ami-images"
  acl    = "private"

  tags = {
    Name = "AMI Images"
  }
}

resource "aws_s3_bucket_public_access_block" "images" {
  bucket = aws_s3_bucket.images.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

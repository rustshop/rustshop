provider "aws" {
  region = local.aws_region
  profile = local.aws_profile
}

terraform {
  backend "s3" {
    # https://github.com/hashicorp/terraform/issues/13022#issuecomment-294262392
    # taken care of by `rustshop-terraform`
  }
}

resource "aws_s3_bucket" "images" {
  bucket = "${local.account_name}-ami"

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

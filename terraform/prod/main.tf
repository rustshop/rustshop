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

# TODO: define lifecycle policy
resource "aws_ecr_repository" "ecr-repo-starter" {
  name                 = "starter"
  image_tag_mutability = "IMMUTABLE"
}

# Give it to CI/CD at some point
# resource "aws_ecr_repository_policy" "ecr" {
#   repository = aws_ecr_repository.ecr.name

#   policy = <<EOF
# {
#     "Version": "2008-10-17",
#     "Statement": [
#         {
#             "Sid": "ci-cd-access-to-${local.account_name}-ecr",
#             "Effect": "Allow",
#             "Principal": "CHANGEME",
#             "Action": [
#                 "ecr:GetDownloadUrlForLayer",
#                 "ecr:BatchGetImage",
#                 "ecr:BatchCheckLayerAvailability",
#                 "ecr:PutImage",
#                 "ecr:InitiateLayerUpload",
#                 "ecr:UploadLayerPart",
#                 "ecr:CompleteLayerUpload",
#                 "ecr:DescribeRepositories",
#                 "ecr:GetRepositoryPolicy",
#                 "ecr:ListImages",
#                 "ecr:DeleteRepository",
#                 "ecr:BatchDeleteImage",
#                 "ecr:SetRepositoryPolicy",
#                 "ecr:DeleteRepositoryPolicy"
#             ]
#         }
#     ]
# }
# EOF
# }

# https://github.com/hashicorp/terraform/issues/13022#issuecomment-294262392
# TODO: use it for something or delete it
# data "terraform_remote_state" "state" {
#   backend = "s3"
#   config = {
#     bucket         = "${local.account_name}-bootstrap-terraform"
#     key            = "state/${local.account_name}.tfstate"
#     dynamodb_table = "${local.account_name}-bootstrap-terraform"
#     region         = "${local.aws_region}"
#     profile        = "${local.account_name}"
#   }
# }

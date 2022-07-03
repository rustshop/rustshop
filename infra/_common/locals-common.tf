locals {
  shopname = var.SHOPNAME
  account_suffix = var.ACCOUNT_SUFFIX
  account_id = var.AWS_ACCOUNT_ID
  account_name = "${var.SHOPNAME}-${local.account_suffix}"
  aws_region = var.AWS_REGION
  aws_profile = var.AWS_PROFILE
}

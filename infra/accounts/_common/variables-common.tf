# Common vars from `.env`
#
# WARNING: This file is shared between all accounts (via symlink)
#

variable "SHOPNAME" {
  description = "Shop Name"
  default     = ""
}

variable "AWS_REGION" {
  description = "AWS Region"
  default     = ""
}

variable "AWS_ACCOUNT_ID_ROOT" {
  description = "AWS Account ID (root)"
  default     = ""
}

variable "AWS_ACCOUNT_ID_ROOT_ROLE" {
  description = "Role to assume for root account - if set"
  default     = ""
}

variable "AWS_ACCOUNT_ID_DEV" {
  description = "AWS Account ID (dev)"
  default     = ""
}

variable "AWS_ACCOUNT_ID_DEV_ROLE" {
  description = "Role to assume for dev account - if set"
  default     = ""
}

variable "AWS_ACCOUNT_ID_PROD" {
  description = "AWS Account ID (prod)"
  default     = ""
}

variable "AWS_ACCOUNT_ID_PROD_ROLE" {
  description = "Role to assume for prod account - if set"
  default     = ""
}

variable "DEFAULT_REGION" {
  description = "Default region to use"
  default     = ""
}

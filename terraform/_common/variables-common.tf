# Common vars from `.env`
#
# WARNING: This file is shared between all accounts (via symlink)
#

variable "SHOPNAME" {
  description = "Shop Name"
  default     = ""
}

variable "ACCOUNT_SUFFIX" {
  description = "Account Name Suffix (eg. root, prod, or dev)"
  default     = ""
}

variable "AWS_REGION" {
  description = "AWS Region"
  default     = ""
}

variable "AWS_ACCOUNT_ID" {
  description = "AWS Account ID"
  default     = ""
}

variable "AWS_PROFILE" {
  description = "AWS Profile"
  default     = ""
}

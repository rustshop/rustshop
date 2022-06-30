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
  description = "AWS Account ID (ROOT)"
  default     = ""
}

variable "AWS_ACCOUNT_ID_DEV" {
  description = "AWS Account ID (DEV)"
  default     = ""
}

variable "AWS_ACCOUNT_ID_PROD" {
  description = "AWS Account ID (PROD)"
  default     = ""
}

variable "DEFAULT_REGION" {
  description = "Default region to use"
  default     = ""
}

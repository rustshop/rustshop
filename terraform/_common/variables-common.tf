# Common vars from `.env`
#
# WARNING: This file is shared between all accounts (via symlink)
#

variable "SHOPNAME" {
  description = "Shop Name"
  default     = ""
}

variable "ACCOUNT_BOOTSTRAP_NAME" {
  description = "Account Name used during bootstrap"
  default     = ""
}

variable "ACCOUNT_BOOTSTRAP_AWS_REGION" {
  description = "AWS Region used during bootstrap"
  default     = ""
}
variable "AWS_REGION" {
  description = "AWS Region"
  default     = ""
}

variable "AWS_PROFILE" {
  description = "AWS Profile"
  default     = ""
}

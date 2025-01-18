variable "name" {
  type = string
}

variable "queue_arn" {
  type = string
}

variable "rule_set" {
  type = string
}

variable "acceptance_required" {
  type    = bool
  default = false
}

variable "timeout" {
  type    = number
  default = 60
}

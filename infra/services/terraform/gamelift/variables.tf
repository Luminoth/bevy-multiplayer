variable "name" {
  type = string
}

variable "image_uri" {
  type = string
}

variable "port_ranges" {
  type = map(object({
    from_port = number
    to_port   = number
    protocol  = optional(string, "UDP")
  }))
}

variable "instance_type" {
  type    = string
  default = "c5.large"
}

variable "operating_system" {
  type    = string
  default = "AMAZON_LINUX_2023"
}

variable "memory_limit" {
  type    = number
  default = 4096
}

variable "cpu_limit" {
  type    = number
  default = 1
}

variable "billing_type" {
  type    = string
  default = "ON_DEMAND"
}

variable "queue_timeout" {
  type    = number
  default = 10
}

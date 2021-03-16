variable "environment" {
  type = string
}

variable "web_domain" {
  type = string
}

variable "files_domain" {
  type = string
}

variable "vpc_cidr_block" {
  type = string
}

variable "elb_subnet" {
  type = string
}

variable "container_subnet" {
  type = string
}

variable "cf_auth_token" {
  type = string
}

variable "region" {
  type = string
}

variable "instance_type" {
  type    = string
  default = "t3.medium"
}

variable "user_data" {
  type    = list(string)
  default = [""]
}

variable "availability_zone" {
  type    = string
  default = "a"
}

variable "ecs_cache_memory" {
  type    = number
  default = 512
}

variable "ecs_cache_cpu" {
  type    = number
  default = 256
}

variable "ecs_cache_image" {
  type = string
}

variable "ecs_db_memory" {
  type    = number
  default = 2048
}

variable "ecs_db_cpu" {
  type    = number
  default = 256
}

variable "ecs_php_memory" {
  type    = number
  default = 768
}

variable "ecs_php_cpu" {
  type    = number
  default = 1024
}

variable "ecs_traefik_memory" {
  type    = number
  default = 512
}

variable "ecs_traefik_cpu" {
  type    = number
  default = 512
}

variable "ecs_traefik_image" {
  type = string
}

variable "letsencrypt_email" {
  type = string
}

variable "redeploy_ecs_on_tf_apply" {
  type = bool
}

variable "route53_access_key" {
  type = string
}

variable "route53_secret_key" {
  type = string
}

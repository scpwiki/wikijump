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
  type      = string
  sensitive = true
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

variable "ecs_api_memory" {
  type    = number
  default = 512
}

variable "ecs_api_cpu" {
  type    = number
  default = 1024
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

variable "ecs_nginx_memory" {
  type    = number
  default = 512
}

variable "ecs_nginx_cpu" {
  type    = number
  default = 512
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

variable "ecs_datadog_cpu" {
  type    = number
  default = 512
}

variable "ecs_datadog_memory" {
  type    = number
  default = 512
}

variable "ecs_traefik_image" {
  type = string
}

variable "letsencrypt_email" {
  type      = string
  sensitive = true
}

variable "redeploy_ecs_on_tf_apply" {
  type = bool
}

variable "route53_access_key" {
  type      = string
  sensitive = true
}

variable "route53_secret_key" {
  type      = string
  sensitive = true
}

variable "api_ratelimit_secret" {
  type      = string
  sensitive = true
}

variable "datadog_api_key" {
  type      = string
  sensitive = true
}

variable "datadog_site" {
  type    = string
  default = "datadoghq.com"
}

variable "s3_asset_bucket_prefix" {
  type    = string
  default = ""
}
variable "s3_elb_log_bucket_prefix" {
  type    = string
  default = ""
}

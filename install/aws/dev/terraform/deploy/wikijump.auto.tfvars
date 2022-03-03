# Meta
environment       = "dev"
web_domain        = "wikijump.dev"
files_domain      = "wjfiles.dev"
region            = "us-east-2"
availability_zone = "a"

# VPC
vpc_cidr_block   = "10.106.0.0/16"
elb_subnet       = "10.106.0.0/24"
container_subnet = "10.106.10.0/24"

# Container Specs
instance_type    = "t3.medium"
ecs_cache_memory = 512
ecs_cache_cpu    = 256
ecs_cache_image  = "memcached:1.6-alpine"

ecs_api_memory = 512
ecs_api_cpu    = 1024

ecs_db_memory = 2048
ecs_db_cpu    = 256

ecs_php_memory = 768
ecs_php_cpu    = 1024

ecs_nginx_memory = 512
ecs_nginx_cpu    = 512

ecs_datadog_memory = 512
ecs_datadog_cpu    = 512

ecs_traefik_memory = 512
ecs_traefik_cpu    = 512
ecs_traefik_image  = "traefik:v2.4"

# S3
# Note: You MUST change these prefixes, they're globally unique (and already in use).
s3_asset_bucket_prefix   = "wikijump-assets"
s3_elb_log_bucket_prefix = "wikijump-elb-logs"

# Misc
letsencrypt_email = "info@wikijump.com"

# If true, ECS will generate a new service with every tf apply, allowing it to
# perhaps pull in a new version of an image. Otherwise, will only generate a new
# service when the service or a related piece is modified.
redeploy_ecs_on_tf_apply = true

# Regarding storing secrets for deployment:
# As we share this repo to the public, but stand up our own installation with
# unique credentials, we store some vars in terraform cloud as terraform vars
# there. Generally speaking some sort of secret store is better than putting
# sensitive keys in a .tfvars file. You need these vars to plan or deploy.
# cf_auth_token        = "12345678-abcd-1234-5678-1234567890ab"
# route53_access_key   = "AEXAMPLEAPIKEY1234"
# route53_secret_key   = "example123/abcdefghijklmnopqrstuvwxyzABC"
# datadog_api_key      = "1234567890abcdef1234567890abcdef"
# api_ratelimit_secret = "1234567890abdefghijklmnopqrstuvwxyz1234567890abdefghijklmnopqrst"

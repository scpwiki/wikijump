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

# Cloudfront/ELB
# The cf_auth_token var isn't *sensitive* as such, but an attacker could add this header value to bypass Cloudfront and hit our load balancer directly.
# So, you may as well store this as a secret, perhaps in Terraform Cloud's vars as a sensitive value so it doesn't show up in tf plans.
# It can be anything, something vaguely random like a GUID will work fine.
# cf_auth_token       = "e421b736-aa0f-4fbf-9965-b6fce423c826"

# Container Specs
instance_type    = "t3.medium"
ecs_cache_memory = 512
ecs_cache_cpu    = 256
ecs_cache_image  = "memcached:1.6-alpine"

ecs_db_memory = 2048
ecs_db_cpu    = 256

ecs_php_memory = 768
ecs_php_cpu    = 1024

ecs_traefik_memory = 512
ecs_traefik_cpu    = 512
ecs_traefik_image  = "traefik:v2.3"

# Misc
letsencrypt_email = "info@wikijump.com"
# If true, ECS will generate a new service with every tf apply, allowing it to perhaps pull in a new version of an image.
# Otherwise, will only generate a new service when the service or a related piece is modified.
redeploy_ecs_on_tf_apply = true

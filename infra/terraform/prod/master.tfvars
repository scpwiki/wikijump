# Meta
environment         = "production"
web_domain          = "wikijump.com"
files_domain        = "wjfiles.com"

# VPC
vpc_cidr_block      = "10.173.0.0/16"
elb_subnet          = "10.173.0.0/24"
container_subnet    = "10.173.10.0/24"
database_subnet     = "10.173.20.0/24"
cache_subnet        = "10.173.30.0/24"

# Cloudfront/ELB
cf_auth_token       = "e421b736-aa0f-4fbf-9965-b6fce423c826"
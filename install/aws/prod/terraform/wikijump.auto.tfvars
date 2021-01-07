# Meta
environment         = "prod"
web_domain          = "wikijump.com"
files_domain        = "wjfiles.com"
region              = "us-east-2"

# VPC
vpc_cidr_block      = "10.173.0.0/16"
elb_subnet          = "10.173.0.0/24"
container_subnet    = "10.173.10.0/24"
database_subnet_a   = "10.173.20.0/24"
database_subnet_b   = "10.173.25.0/24"
cache_subnet        = "10.173.30.0/24"

# Cloudfront/ELB
# The cf_auth_token var isn't *sensitive* as such, but an attacker could add this header value to bypass Cloudfront and hit our load balancer directly.
# So, you may as well store this as a secret, perhaps in Terraform Cloud's vars as a sensitive value so it doesn't show up in tf plans.
# It can be anything, something vaguely random will work fine like a GUID.
# cf_auth_token       = "e421b736-aa0f-4fbf-9965-b6fce423c826"

# Elasticache
cache_num_nodes     = 1
cache_ec2_size      = "cache.t3.micro"

# RDS
# You should pull this value from some secret manager, whether that's Vault, SSM Parameter Store, or Terraform Cloud's Sensitive variable storage.
# rds_master_password   = (stored in TF cloud)

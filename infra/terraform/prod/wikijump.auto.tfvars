# Meta
environment         = "prod"
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

# Elasticache
cache_num_nodes     = 1
cache_ec2_size      = "cache.t3.micro"

# RDS
# Note: We deploy with Terraform Cloud and so have certain secrets (RDS master password) stored as a Terraform Variable there.
# rds_master_password   = (stored in TF cloud)
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 3.0"
    }
  }
}

# Configure the AWS Provider
provider "aws" {
  region = var.region
}

# us-east-1 region, used to get ACM to attach correctly to CloudFront
provider "aws" {
  region = "us-east-1"
  alias  = "us-east-1"
}

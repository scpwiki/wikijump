locals {
    environment = "dev"
    region = "us-east-2"
}

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
  region = local.region
}


resource "aws_ssm_parameter" "WEB_ECR_URL" {
  name  = "wikijump-${local.environment}-WEB_ECR_URL"
  type  = "String"
  value = aws_ecr_repository.web_ecr.repository_url
}

resource "aws_ssm_parameter" "DB_ECR_URL" {
  name  = "wikijump-${local.environment}-DB_ECR_URL"
  type  = "String"
  value = aws_ecr_repository.db_ecr.repository_url
}

resource "aws_ecr_repository" "web_ecr" {
  name = "wikijump-${local.environment}/php-fpm"
  encryption_configuration {
    encryption_type = "KMS"
  }
  image_scanning_configuration {
    scan_on_push = true
  }
}

resource "aws_ecr_repository" "db_ecr" {
  name = "wikijump-${local.environment}/postgres"
  encryption_configuration {
    encryption_type = "KMS"
  }
  image_scanning_configuration {
    scan_on_push = true
  }
}

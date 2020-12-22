resource "aws_ssm_parameter" "URL_DOMAIN" {
  name  = "wikijump-${var.environment}-URL_DOMAIN"
  type  = "String"
  value = var.web_domain
}

resource "aws_ssm_parameter" "URL_UPLOAD_DOMAIN" {
  name  = "wikijump-${var.environment}-URL_UPLOAD_DOMAIN"
  type  = "String"
  value = var.files_domain
}

resource "aws_ssm_parameter" "WEB_ECR_URL" {
  name  = "wikijump-${var.environment}-WEB_ECR_URL"
  type  = "String"
  value = aws_ecr_repository.web_ecr.repository_url
}

resource "aws_ssm_parameter" "DB_ECR_URL" {
  name  = "wikijump-${var.environment}-DB_ECR_URL"
  type  = "String"
  value = aws_ecr_repository.db_ecr.repository_url
}

data "aws_ssm_parameter" "ecs_ami" {
  name = "/aws/service/ecs/optimized-ami/amazon-linux-2/recommended/image_id"
}

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

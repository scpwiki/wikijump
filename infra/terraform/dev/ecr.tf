resource "aws_ecr_repository" "web_ecr" {
  name = "wikijump-${var.environment}/php-fpm"
  encryption_configuration {
    encryption_type = "KMS"
  }
  image_scanning_configuration {
    scan_on_push = true
  }
}

resource "aws_ecr_repository" "db_ecr" {
  name = "wikijump-${var.environment}/postgres"
  encryption_configuration {
    encryption_type = "KMS"
  }
  image_scanning_configuration {
    scan_on_push = true
  }
}




resource "aws_ecr_repository" "wikijump_ecr" {
    name                        = "wikijump-${var.environment}"
    encryption_configuration    = "KMS"

    image_scanning_configuration {
        scan_on_push            = true
  }
}
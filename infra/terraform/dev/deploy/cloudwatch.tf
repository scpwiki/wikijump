resource "aws_cloudwatch_log_group" "cache" {
  name              = "ecs/cache-${var.environment}"
  retention_in_days = "7"
}

resource "aws_cloudwatch_log_group" "database" {
  name              = "ecs/database-${var.environment}"
  retention_in_days = "7"
}

resource "aws_cloudwatch_log_group" "php-fpm" {
  name              = "ecs/php-fpm-${var.environment}"
  retention_in_days = "7"
}

resource "aws_cloudwatch_log_group" "traefik" {
  name              = "ecs/traefik-${var.environment}"
  retention_in_days = "7"
}

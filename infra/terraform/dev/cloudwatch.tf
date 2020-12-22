resource "aws_cloudwatch_log_group" "cache" {
  name              = "ecs/cache"
  retention_in_days = "7"
}
resource "aws_cloudwatch_log_group" "database" {
  name              = "ecs/database"
  retention_in_days = "7"
}
resource "aws_cloudwatch_log_group" "php-fpm" {
  name              = "ecs/php-fpm"
  retention_in_days = "7"
}
resource "aws_cloudwatch_log_group" "traefik" {
  name              = "ecs/traefik"
  retention_in_days = "7"
}

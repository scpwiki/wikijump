resource "aws_s3_bucket" "elb_logs" {
  bucket = "wikijump-elb-logs-${var.environment}"
  acl    = "log-delivery-write"
}

resource "aws_s3_bucket" "wikijump_assets" {
  bucket = "wikijump-assets-${var.environment}"
  acl    = "public-read"
}

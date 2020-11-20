resource "aws_s3_bucket" "elb_logs" {
    bucket  = "wikijump-elb-logs-${var.environment}"
    acl     =  "log-delivery-write"
}
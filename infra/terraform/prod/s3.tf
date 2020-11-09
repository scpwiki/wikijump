resource "aws_s3_bucket" "elb_logs" {
    bucket  = "wikijump_elb_logs"
    acl     =  "log-delivery-write"
}
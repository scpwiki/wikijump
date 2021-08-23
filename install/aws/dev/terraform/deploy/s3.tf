resource "aws_s3_bucket" "elb_logs" {
  bucket = "${var.s3_elb_log_bucket_prefix}-${var.environment}"
  acl    = "log-delivery-write"
}

resource "aws_s3_bucket" "wikijump_assets" {
  bucket = "${var.s3_asset_bucket_prefix}-${var.environment}"
  acl    = "public-read"

  cors_rule {
    allowed_methods = [
      "GET",
      "POST",
      "HEAD",
      "DELETE",
      "PUT"
    ]
    allowed_origins = ["*"] # TODO: Tighten up in WJ-783
    allowed_headers = ["*"]
    expose_headers  = []
    max_age_seconds = 0
  }
  lifecycle_rule {
    enabled = true
    id      = "Expire temporary uploads after 1 day"
    prefix  = "livewire-tmp"
    tags    = {}

    expiration {
      days                         = 1
      expired_object_delete_marker = false
    }
  }
}

data "aws_caller_identity" "current" {}

resource "aws_s3_bucket_policy" "elb_log_policy" {
  bucket = aws_s3_bucket.elb_logs.id
  policy = <<POLICY
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AWSLogDeliveryWrite",
      "Effect": "Allow",
      "Principal": {
        "Service": "delivery.logs.amazonaws.com"
      },
      "Action": "s3:PutObject",
      "Resource": "${aws_s3_bucket.elb_logs.arn}/prefix/AWSLogs/${data.aws_caller_identity.current.account_id}/*",
      "Condition": {
        "StringEquals": {
          "s3:x-amz-acl": "bucket-owner-full-control"
        }
      }
    },
    {
      "Sid": "AWSLogDeliveryAclCheck",
      "Effect": "Allow",
      "Principal": {
        "Service": "delivery.logs.amazonaws.com"
      },
      "Action": "s3:GetBucketAcl",
      "Resource": "${aws_s3_bucket.elb_logs.arn}"
    }
  ]
}
POLICY
}

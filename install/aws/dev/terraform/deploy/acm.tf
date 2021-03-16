resource "aws_acm_certificate" "cf_wildcard_cert" {
  domain_name = var.files_domain
  subject_alternative_names = [
    "*.${var.files_domain}"
  ]
  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  # We hardcode us-east-1 here because CloudFront will look for this cert in us-east-1.
  # See: https://docs.aws.amazon.com/acm/latest/userguide/acm-regions.html
  provider = aws.us-east-1
}

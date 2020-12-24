resource "aws_route53_zone" "primary" {
  name = var.web_domain
}

resource "aws_route53_zone" "files" {
  name = var.files_domain
}

resource "aws_route53_record" "primary_record" {
  zone_id = aws_route53_zone.primary.zone_id
  name    = var.web_domain
  type    = "A"
  alias {
    name                   = aws_lb.wikijump_elb.dns_name
    zone_id                = aws_lb.wikijump_elb.zone_id
    evaluate_target_health = true
  }

  allow_overwrite = true
}

resource "aws_route53_record" "primary_wildcard" {
  zone_id         = aws_route53_zone.primary.zone_id
  name            = "*.${var.web_domain}"
  type            = "CNAME"
  ttl             = "300"
  records         = [var.web_domain]
  allow_overwrite = true
}

resource "aws_route53_record" "files_record" {
  zone_id = aws_route53_zone.files.zone_id
  name    = var.files_domain
  type    = "A"
  alias {
    name                   = aws_cloudfront_distribution.wikijump_cf_distro.domain_name
    zone_id                = aws_cloudfront_distribution.wikijump_cf_distro.hosted_zone_id
    evaluate_target_health = true
  }
  allow_overwrite = true
}

resource "aws_route53_record" "files_wildcard" {
  zone_id         = aws_route53_zone.files.zone_id
  name            = "*.${var.files_domain}"
  type            = "CNAME"
  ttl             = "300"
  records         = [var.files_domain]
  allow_overwrite = true
}

resource "aws_route53_record" "acm_validation" {
  for_each = {
    for row in aws_acm_certificate.cf_wildcard_cert.domain_validation_options : row.domain_name => {
      name   = row.resource_record_name
      record = row.resource_record_value
      type   = row.resource_record_type
    }
  }
  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = aws_route53_zone.files.zone_id
}

resource "aws_acm_certificate_validation" "acm_validation" {
  certificate_arn         = aws_acm_certificate.cf_wildcard_cert.arn
  validation_record_fqdns = [for record in aws_route53_record.acm_validation : record.fqdn]

  # We hardcode us-east-1 here because CloudFront will look for this cert in us-east-1.
  # See: https://docs.aws.amazon.com/acm/latest/userguide/acm-regions.html
  provider = aws.us-east-1
}



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
    name                   = aws_lb.wikijump_elb.dns_name
    zone_id                = aws_lb.wikijump_elb.zone_id
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

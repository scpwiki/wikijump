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
  ttl     = "300"
  records = [aws_eip.wikijump_elb.public_ip]
}

resource "aws_route53_record" "primary_wildcard" {
  zone_id = aws_route53_zone.primary.zone_id
  name    = "*.${var.web_domain}"
  type    = "CNAME"
  ttl     = "300"
  records = [var.web_domain]
}

resource "aws_route53_record" "files_record" {
  zone_id = aws_route53_zone.files.zone_id
  name    = var.files_domain
  type    = "A"
  ttl     = "300"
  records = [aws_eip.wikijump_elb.public_ip]
}

resource "aws_route53_record" "files_wildcard" {
  zone_id = aws_route53_zone.files.zone_id
  name    = "*.${var.files_domain}"
  type    = "CNAME"
  ttl     = "300"
  records = [var.files_domain]
}
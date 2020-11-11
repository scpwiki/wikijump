resource "aws_cloudfront_distribution" "wikijump_cf_distro" {
    enabled                 = true
    is_ipv6_enabled         = true
    default_root_object     = "index.php"

    aliases                 = [var.web_domain, var.files_domain]

    origin {
        domain_name         = aws_lb.wikijump_elb.dns_name
        origin_id           = "wikijump_elb"
        custom_headers      = [
            {
            name            = "X-CLOUDFRONT-WIKIJUMP-AUTH",
            value           = var.cf_auth_token
            }
        ]
    }

    default_cache_behavior {
    allowed_methods         = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods          = ["GET", "HEAD"]
    target_origin_id        = "wikijump_elb"

    forwarded_values {
      query_string          = true

      cookies {
        forward             = "all"
      }
    }

    viewer_protocol_policy  = "redirect-to-https"
    min_ttl                 = 0
    default_ttl             = 60
    compress                = true
    max_ttl                 = 60
    }

    viewer_certificate {
        acm_certificate_arn = aws_acm_certificate.cf_wildcard_cert.arn
  }
}
# resource "aws_cloudfront_distribution" "wikijump_cf_distro" {
#     enabled                 = true
#     is_ipv6_enabled         = true
#     default_root_object     = "index.php"

#     aliases                 = [var.files_domain]

#     origin {
#         domain_name         = aws_lb.wikijump_elb.dns_name
#         origin_id           = "wikijump_elb"
#         custom_header {
#             name            = "X-CLOUDFRONT-WIKIJUMP-AUTH"
#             value           = var.cf_auth_token
#         }
#     }

    # origin {
    #   domain_name = aws_s3_bucket.wikijump_assets.bucket_domain_name
    #   origin_id = "wikijump_s3"
    # }

    # restrictions {
    #   geo_restriction {
    #     restriction_type  = "none"
    #   }
    # }

#  ordered_cache_behavior {
#     path_pattern     = "/local--files/*"
#     allowed_methods  = ["GET", "HEAD", "OPTIONS"]
#     cached_methods   = ["GET", "HEAD", "OPTIONS"]
#     target_origin_id = "wikijump_s3"

#     forwarded_values {
#       query_string = false
#       headers      = ["Origin"]

#       cookies {
#         forward = "none"
#       }
#     }

#     min_ttl                = 0
#     default_ttl            = 86400
#     max_ttl                = 31536000
#     compress               = true
#     viewer_protocol_policy = "redirect-to-https"
#   }

#   # Cache behavior with precedence 1
#   ordered_cache_behavior {
#     path_pattern     = "/local--code/*"
#     allowed_methods  = ["GET", "HEAD", "OPTIONS"]
#     cached_methods   = ["GET", "HEAD"]
#     target_origin_id = "wikijump_elb"

#     forwarded_values {
#       query_string = false

#       cookies {
#         forward = "all"
#       }
#     }

#     min_ttl                = 0
#     default_ttl            = 3600
#     max_ttl                = 86400
#     compress               = true
#     viewer_protocol_policy = "redirect-to-https"
#   }

#    default_cache_behavior {
#     allowed_methods         = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
#     cached_methods          = ["GET", "HEAD"]
#     target_origin_id        = "wikijump_elb"

#     forwarded_values {
#       query_string          = true

#       cookies {
#         forward             = "all"
#       }
#     }

#     viewer_protocol_policy  = "redirect-to-https"
#     min_ttl                 = 0
#     default_ttl             = 60
#     compress                = true
#     max_ttl                 = 60
#     }

#     viewer_certificate {
#         acm_certificate_arn = aws_acm_certificate.cf_wildcard_cert.arn
#         ssl_support_method = "sni-only"
#   }
# }

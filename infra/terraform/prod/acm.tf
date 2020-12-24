resource "aws_acm_certificate" "cf_wildcard_cert" {
    domain_name                 = var.web_domain
    subject_alternative_names   = [
        var.files_domain,
        "*.${var.files_domain}"
    ]
    validation_method           = "DNS"

    lifecycle {
        create_before_destroy   = true
    }
}

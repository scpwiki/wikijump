# ELB

resource "aws_lb" "wikijump_elb" {
    name                        = "wikijump_public_elb_${var.environment}"
    internal                    = false
    load_balancer_type          = "application"
    security_groups             = [aws_security_group.elb_sg.id]
    subnet_mapping {
        subnet_id               = aws_subnet.elb_subnet.id
        allocation_id           = aws_eip.elb_eip.id
    }
    
    # Enable this once stable.
    enable_deletion_protection  = false

    access_logs {
        bucket                  = aws_s3_bucket.elb_logs.bucket
        prefix                  = var.environment
        # Logging is currently OFF
        enabled                 = false
    }
}

resource "aws_lb_target_group" "elb_target_group" {
    name        = "wikijump_tg_80_${var.environment}"
    port        = 80
    protocol    = "HTTP"
    vpc_id      = aws_vpc.wikijump_vpc.id
    health_check {
        enabled = false
        path    = "/heartbeat.php"
        matcher = "200"
    }
}

resource "aws_lb_listener" "elb_listener" {
    load_balancer_arn       = aws_lb.wikijump_elb.arn
    port                    = 80
    default_action {
        type                = forward
        forward {
            target_group    {
                arn    = aws_lb_target_group.elb_target_group.arn
            }
            stickiness {
                enabled         = false
            }
        }
    }
}

resource "aws_lb_listener_rule" "cloudfront_header_check" {
    listener_arn            = aws_lb_listener.elb_listener.arn
    priority                = 100

    action {
        type                = "forward"
        target_group_arn    = aws_lb_target_group.elb_target_group.arn
    }

    condition {
        http_header {
        http_header_name    = "X-CLOUDFRONT-WIKIJUMP-AUTH"
        values              = var.cf_auth_token
        }
    }
}

resource "aws_lb_listener_rule" "fallback" {
    listener_arn            = aws_lb_listener.elb_listener.arn
    priority                = 999

    action {
        type                = "fixed-response"
        fixed_response {
            content_type    = "text/plain"
            message_body    = "CloudFront Token Missing"
            status_code     = "400"
        } 
    }

}

# Security Group

resource "aws_security_group" "elb_sg" {
    name            = "elb_sg_${var.environment}"
    description     = "Allow 80 inbound"

    ingress {
        description = "HTTP"
        from_port   = 80
        to_port     = 80
        protocol    = "tcp"
        cidr_blocks = ["0.0.0.0/0"]  # Note we will add a header to invalidate requests from other than behind cloudfront.
    }

    egress {
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }
}

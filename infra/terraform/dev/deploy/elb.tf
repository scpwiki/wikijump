# ELB

resource "aws_lb" "wikijump_elb" {
  name               = "wikijump-public-elb-${var.environment}"
  internal           = false
  load_balancer_type = "network"
  subnet_mapping {
    subnet_id     = aws_subnet.elb_subnet.id
    allocation_id = aws_eip.elb_eip.id
  }
  # TODO: IPv6 support for everything.
  # ip_address_type = "dualstack"
  enable_deletion_protection = true

  access_logs {
    bucket  = aws_s3_bucket.elb_logs.bucket
    prefix  = var.environment
    enabled = false
  }
}

resource "aws_lb_target_group" "elb_target_group_80" {
  name        = "wikijump-tg-80-${var.environment}"
  port        = 80
  protocol    = "TCP"
  vpc_id      = aws_vpc.wikijump_vpc.id
  target_type = "instance"
  deregistration_delay = 3
  health_check {
    enabled = true
    port    = 8081
    path    = "/ping"
  }
}

resource "aws_lb_target_group" "elb_target_group_443" {
  name        = "wikijump-tg-443-${var.environment}"
  port        = 443
  protocol    = "TCP"
  vpc_id      = aws_vpc.wikijump_vpc.id
  target_type = "instance"
  deregistration_delay = 3
  health_check {
    enabled = true
    port    = 8081
    path    = "/ping"
  }
}

resource "aws_lb_listener" "elb_listener_80" {
  load_balancer_arn = aws_lb.wikijump_elb.arn
  port              = 80
  protocol          = "TCP"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.elb_target_group_80.arn
  }
}

resource "aws_lb_listener" "elb_listener_443" {
  load_balancer_arn = aws_lb.wikijump_elb.arn
  port              = 443
  protocol          = "TCP"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.elb_target_group_443.arn
  }
}

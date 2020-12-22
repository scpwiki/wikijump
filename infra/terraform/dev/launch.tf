data "template_cloudinit_config" "config" {
  gzip          = false
  base64_encode = true

  part {
    content_type = "text/x-shellscript"
    content      = <<EOT
#!/bin/bash
echo ECS_CLUSTER="wikijump-${var.environment}" >> /etc/ecs/ecs.config
echo ECS_ENABLE_CONTAINER_METADATA=true >> /etc/ecs/ecs.config
echo ECS_ENABLE_SPOT_INSTANCE_DRAINING=true >> /etc/ecs/ecs.config
EOT
  }

  dynamic "part" {
    for_each = var.user_data
    content {
      content_type = "text/x-shellscript"
      content      = part.value
    }
  }
}

data "aws_ssm_parameter" "ecs_ami" {
  name = "/aws/service/ecs/optimized-ami/amazon-linux-2/recommended/image_id"
}

resource "aws_launch_template" "node" {
  name_prefix            = "ecs_node_"
  image_id               = data.aws_ssm_parameter.ecs_ami.value
  instance_type          = var.instance_type
  vpc_security_group_ids = [aws_security_group.ecs_nodes.id]
  user_data              = data.template_cloudinit_config.config.rendered
  update_default_version = true

  iam_instance_profile {
    name = aws_iam_instance_profile.ecs_node.name
  }
}

resource "aws_security_group" "ecs_nodes" {
  name   = "ECS nodes for Wikijump dev"
  vpc_id = aws_vpc.wikijump_vpc.id
}

resource "aws_security_group_rule" "ingress" {
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = [var.container_subnet, var.elb_subnet]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "ingress"
}

resource "aws_security_group_rule" "egress" {
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "egress"
}

resource "aws_iam_instance_profile" "ecs_node" {
  name = "wikijump-ecs-ec2-dev"
  role = aws_iam_role.ec2_instance_role.name
}

resource "aws_iam_role" "ec2_instance_role" {
  assume_role_policy = data.aws_iam_policy_document.ec2_instance_assume_role_policy.json
  name               = "wikijump-ec2-role-dev"
}

data "aws_iam_policy_document" "ec2_instance_assume_role_policy" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

data "template_cloudinit_config" "config" {
  gzip          = false
  base64_encode = true

  part {
    content_type = "text/x-shellscript"
    content      = <<EOT
#!/bin/bash
set -eu
echo ECS_CLUSTER=wikijump-${var.environment} >> /etc/ecs/ecs.config
echo ECS_ENABLE_CONTAINER_METADATA=true >> /etc/ecs/ecs.config
echo ECS_ENABLE_SPOT_INSTANCE_DRAINING=true >> /etc/ecs/ecs.config
echo ECS_CONTAINER_STOP_TIMEOUT=3s >> /etc/etc/ecs.config
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

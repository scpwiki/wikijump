resource "aws_ecs_cluster" "wikijump-ecs" {
  name               = "wikijump-${var.environment}"
  capacity_providers = [aws_ecs_capacity_provider.asg.name]

  default_capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.asg.name
    weight            = 1
  }
}

resource "aws_autoscaling_group" "ecs_nodes" {
  name_prefix           = "CLUSTER_NODES_"
  desired_capacity      = 1
  max_size              = 2
  min_size              = 1
  vpc_zone_identifier   = [aws_subnet.container_subnet.id]
  protect_from_scale_in = false

  mixed_instances_policy {
    instances_distribution {
      on_demand_percentage_above_base_capacity = 0
    }
    launch_template {
      launch_template_specification {
        launch_template_id = aws_launch_template.node.id
        version            = "$Latest"
      }

      override {
        instance_type     = var.instance_type
        weighted_capacity = 1
      }
    }
  }

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [aws_launch_template.node]
}

resource "aws_ecs_capacity_provider" "asg" {
  name = aws_autoscaling_group.ecs_nodes.name

  auto_scaling_group_provider {
    auto_scaling_group_arn         = aws_autoscaling_group.ecs_nodes.arn
    managed_termination_protection = "DISABLED"

    managed_scaling {
      maximum_scaling_step_size = 1
      minimum_scaling_step_size = 1
      status                    = "DISABLED"
      target_capacity           = 1
    }
  }
}

resource "aws_ecs_task_definition" "wikijump_task" {
  family                   = "wikijump-${var.environment}-ec2"
  container_definitions    = "[${module.cache.json_map_encoded},${module.database.json_map_encoded},${module.php-fpm.json_map_encoded},${module.reverse-proxy.json_map_encoded}]"
  requires_compatibilities = ["EC2"]
  network_mode             = "bridge"
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn
  volume {
    name      = "docker-socket"
    host_path = "/var/run/docker.sock"
  }
  volume {
    name = "letsencrypt"

    efs_volume_configuration {
      file_system_id     = data.aws_ssm_parameter.TRAEFIK_EFS_ID.value
      transit_encryption = "ENABLED"
      root_directory     = "/letsencrypt"
      authorization_config {
        access_point_id = aws_efs_access_point.access.id
      }
    }
  }
}

resource "aws_ecs_service" "wikijump" {
  name                               = "wikijump-${var.environment}-svc"
  cluster                            = aws_ecs_cluster.wikijump-ecs.id
  task_definition                    = aws_ecs_task_definition.wikijump_task.arn
  deployment_minimum_healthy_percent = 0
  deployment_maximum_percent         = 200
  desired_count                      = 1 # This will be a var as we grow
  force_new_deployment               = var.redeploy_ecs_on_tf_apply
  load_balancer {
    target_group_arn = aws_lb_target_group.elb_target_group_443.arn
    container_name   = "reverse-proxy"
    container_port   = 443
  }
  load_balancer {
    target_group_arn = aws_lb_target_group.elb_target_group_80.arn
    container_name   = "reverse-proxy"
    container_port   = 80
  }
}

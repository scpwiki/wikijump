module "ecs_cluster" {
  source       = "github.com/jetbrains-infra/terraform-aws-ecs-cluster?ref=v0.4.8" // see https://github.com/jetbrains-infra/terraform-aws-ecs-cluster/releases
  cluster_name = "wikijump-dev"
  spot         = true
  instance_types = {
    "t3.medium"  = 1
  }
  target_capacity = 1

  // subnets with ALB and bastion host e.g..
  trusted_cidr_blocks = [
    var.elb_subnet,
    var.container_subnet
  ]

  subnets_ids = [
    aws_subnet.container_subnet.id
  ]

  tags = {
    Stack     = "Dev",
    Terraform = "true"
  }
}


resource "aws_ecs_task_definition" "wikijump_task" {
  family                   = "wikijump-${var.environment}-ec2"
  container_definitions    = file("task-definitions/dev-ec2.json")
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
      file_system_id = aws_efs_file_system.traefik_efs.id
      root_directory = "/letsencrypt"
    }
  }
}


resource "aws_ecs_service" "wikijump" {
  name                 = "wikijump-${var.environment}-svc"
  cluster              = module.ecs_cluster.id
  task_definition      = aws_ecs_task_definition.wikijump_task.arn
  desired_count        = 1 # This will be a var as we grow
  force_new_deployment = true
  load_balancer {
    target_group_arn = aws_lb_target_group.elb_target_group_443.arn
    container_name   = "reverse-proxy"
    container_port   = 443
  }
}




module "cache" {
  source = "github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

  container_name               = "cache"
  container_image              = var.ecs_cache_image
  container_memory_reservation = var.ecs_cache_memory / 4
  essential                    = true

  log_configuration = {
    logDriver = "awslogs"
    options = {
      "awslogs-group"         = "ecs/cache-${var.environment}"
      "awslogs-region"        = var.region
      "awslogs-stream-prefix" = "ecs"
    }
  }
}

module "database" {
  source = "github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

  container_name               = "database"
  container_image              = "${data.aws_ssm_parameter.DB_ECR_URL.value}:develop"
  container_memory_reservation = var.ecs_db_memory / 4
  essential                    = true

  log_configuration = {
    logDriver = "awslogs"
    options = {
      "awslogs-group"         = "ecs/database-${var.environment}"
      "awslogs-region"        = var.region
      "awslogs-stream-prefix" = "ecs"
    }
  }
}

module "php-fpm" {
  source = "github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

  container_name               = "php-fpm"
  container_image              = "${data.aws_ssm_parameter.WEB_ECR_URL.value}:develop"
  container_memory_reservation = var.ecs_php_memory / 4
  essential                    = true

  log_configuration = {
    logDriver = "awslogs"
    options = {
      "awslogs-group"         = "ecs/php-fpm-${var.environment}"
      "awslogs-region"        = var.region
      "awslogs-stream-prefix" = "ecs"
    }
  }

  links = ["cache:cache", "database:database"]

  secrets = [
    {
      name      = "URL_DOMAIN"
      valueFrom = "wikijump-dev-URL_DOMAIN"
    },
    {
      name      = "URL_UPLOAD_DOMAIN"
      valueFrom = "wikijump-dev-URL_UPLOAD_DOMAIN"
    },
    {
      name      = "DB_HOST"
      valueFrom = "wikijump-dev-DB_HOST"
    }
  ]

  docker_labels = {
    "traefik.enable"                                = "true"
    "traefik.http.routers.php-fpm.rule"             = "Host(`${var.web_domain}`,`www.${var.web_domain}`)"
    "traefik.http.routers.php-fpm.tls"              = "true"
    "traefik.http.routers.php-fpm.tls.certresolver" = "mytlschallenge"
  }

  healthcheck = {
    command     = ["CMD-SHELL", "curl -f http://localhost"]
    interval    = 30
    timeout     = 5
    retries     = 3
    startPeriod = 15
  }
}

module "reverse-proxy" {
  source = "github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

  container_name               = "reverse-proxy"
  container_image              = var.ecs_traefik_image
  container_memory_reservation = var.ecs_traefik_memory / 4
  essential                    = true

  log_configuration = {
    logDriver = "awslogs"
    options = {
      "awslogs-group"         = "ecs/traefik-${var.environment}"
      "awslogs-region"        = var.region
      "awslogs-stream-prefix" = "ecs"
    }
  }

  links = ["php-fpm:php-fpm"]

  port_mappings = [
    {
      containerPort = 8081
      hostPort      = 8081
      protocol      = "tcp"
    },
    {
      containerPort = 443
      hostPort      = 443
      protocol      = "tcp"
    },
    {
      containerPort = 80
      hostPort      = 80
      protocol      = "tcp"
    }
  ]

  command = [
    "--providers.docker",
    "--entrypoints.web.address=:80",
    "--entrypoints.web.http.redirections.entryPoint.to=web-secure",
    "--entrypoints.web.http.redirections.entryPoint.scheme=https",
    "--entrypoints.web.http.redirections.entrypoint.permanent=true",
    "--entrypoints.web-secure.address=:443",
    "--certificatesresolvers.mytlschallenge.acme.httpchallenge.entrypoint=web",
    "--certificatesresolvers.mytlschallenge.acme.email=${var.letsencrypt_email}",
    "--certificatesresolvers.mytlschallenge.acme.storage=/letsencrypt/acme.json",
    "--ping.entrypoint=ping",
    "--entrypoints.ping.address=:8081"
  ]
  mount_points = [
    {
      sourceVolume  = "docker-socket"
      containerPath = "/var/run/docker.sock"
    },
    {
      sourceVolume  = "letsencrypt"
      containerPath = "/letsencrypt"
    }
  ]

  container_depends_on = [
    {
      containerName = "php-fpm"
      condition     = "HEALTHY"
    }
  ]
}

output "cache_json" {
  description = "Container definition in JSON format"
  value       = module.cache.json_map_encoded_list
}

output "database_json" {
  description = "Container definition in JSON format"
  value       = module.database.json_map_encoded_list
}

output "php-fpm_json" {
  description = "Container definition in JSON format"
  value       = module.php-fpm.json_map_encoded_list
}

output "reverse-proxy_json" {
  description = "Container definition in JSON format"
  value       = module.reverse-proxy.json_map_encoded_list
}

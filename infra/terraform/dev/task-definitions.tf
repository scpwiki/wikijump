module "cache" {
    source="github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

    container_name = "cache"
    container_image = "memcached:1.6-alpine"
    container_memory = 512
    container_cpu = 256
    essential = true

    log_configuration = {
        logDriver = "awslogs"
        options = {
            "awslogs-group" = "ecs/cache"
            "awslogs-region" = "us-east-2"
            "awslogs-stream-prefix" = ""
        }
    }
}

module "database" {
    source="github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

    container_name = "database"
    container_image = "${aws_ecr_repository.db_ecr.repository_url}:develop"
    container_memory = 2048
    container_cpu = 256
    essential = true

    log_configuration = {
        logDriver = "awslogs"
        options = {
            "awslogs-group" = "ecs/database"
            "awslogs-region" = "us-east-2"
            "awslogs-stream-prefix" = ""
        }
    }
}

module "php-fpm" {
    source="github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

    container_name = "php-fpm"
    container_image = "${aws_ecr_repository.web_ecr.repository_url}:develop"
    container_memory = 768
    container_cpu = 1024
    essential = true

    log_configuration = {
        logDriver = "awslogs"
        options = {
            "awslogs-group" = "ecs/php-fpm"
            "awslogs-region" = "us-east-2"
            "awslogs-stream-prefix" = ""
        }
    }

    links = ["cache:cache", "database:database"]

    secrets = [
        {
            name = "URL_DOMAIN"
            valueFrom = "wikijump-dev-URL_DOMAIN"
        },
        {
            name = "URL_UPLOAD_DOMAIN"
            valueFrom = "wikijump-dev-URL_UPLOAD_DOMAIN"
        }
    ]

    docker_labels = {
        "traefik.enable" = "true"
        "traefik.http.routers.php-fpm.rule" = "Host(`wikijump.dev`,`www.wikijump.dev`,`wjfiles.dev`,`www.wjfiles.dev`)"
        "traefik.http.routers.php-fpm.tls" = "true"
        "traefik.http.routers.php-fpm.tls.certresolver" = "mytlschallenge"
    }

    healthcheck = {
        command = ["CMD-SHELL", "curl -f http://localhost || exit 1"]
        interval = 30
        timeout = 5
        retries = 3
        startPeriod = 15
    }
}

module "reverse-proxy" {
    source="github.com/cloudposse/terraform-aws-ecs-container-definition?ref=0.46.0"

    container_name = "reverse-proxy"
    container_image = "traefik:v2.3"
    container_memory = 512
    container_cpu = 512
    essential = true

    log_configuration = {
        logDriver = "awslogs"
        options = {
            "awslogs-group" = "ecs/traefik"
            "awslogs-region" = "us-east-2"
            "awslogs-stream-prefix" = ""
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
                    "--certificatesresolvers.mytlschallenge.acme.tlschallenge=true",
                    "--certificatesresolvers.mytlschallenge.acme.email=info@wikijump.com",
                    "--certificatesresolvers.mytlschallenge.acme.storage=/letsencrypt/acme.json",
                    "--ping.entrypoint=ping",
                    "--entrypoints.ping.address=:8081"
                ]
    mount_points = [
        {
            sourceVolume = "docker-socket"
            containerPath = "/var/run/docker.sock"
            readOnly = "true"
        },
        {
            sourceVolume = "letsencrypt"
            containerPath = "/letsencrypt"
        }
    ]

    container_depends_on = [
        {
            containerName = "php-fpm"
            condition = "HEALTHY"
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

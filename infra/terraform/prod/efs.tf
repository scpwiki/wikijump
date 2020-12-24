resource "aws_efs_file_system" "traefik_efs" {
    creation_token = "traefik-certstore-${var.environment}"
    encrypted = true
}


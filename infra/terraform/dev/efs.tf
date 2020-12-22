resource "aws_efs_file_system" "traefik_efs" {
  creation_token = "traefik-certstore-${var.environment}"
  encrypted      = true
}

resource "aws_efs_access_point" "access" {
  file_system_id = aws_efs_file_system.traefik_efs.id
}

resource "aws_efs_mount_target" "traefik_mount" {
  file_system_id  = aws_efs_file_system.traefik_efs.id
  subnet_id       = aws_subnet.container_subnet.id
  security_groups = [aws_security_group.ecs_nodes.id]
}

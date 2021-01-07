resource "aws_efs_access_point" "access" {
  file_system_id = data.aws_ssm_parameter.TRAEFIK_EFS_ID.value
}

resource "aws_efs_mount_target" "traefik_mount" {
  file_system_id  = data.aws_ssm_parameter.TRAEFIK_EFS_ID.value
  subnet_id       = aws_subnet.container_subnet.id
  security_groups = [aws_security_group.ecs_nodes.id]
}

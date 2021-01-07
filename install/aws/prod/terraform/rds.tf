resource "aws_rds_cluster" "wikijump_aurora" {
  engine        = "aurora-postgresql"
  # engine_version  = "10.12_postgres_aurora"
  engine_mode = "serverless"

  scaling_configuration {
    auto_pause               = true
    max_capacity             = 4
    min_capacity             = 2
    seconds_until_auto_pause = 300
    timeout_action           = "RollbackCapacityChange"
  }

  db_subnet_group_name  = aws_db_subnet_group.wikijump_aurora_subnet.name
  master_username = "aurora"
  master_password = var.rds_master_password
  skip_final_snapshot  = true
}

resource "aws_db_subnet_group" "wikijump_aurora_subnet" {
  name       = "wikijump-aurora-${var.environment}"
  subnet_ids = [aws_subnet.database_subnet_a.id, aws_subnet.database_subnet_b.id]
}

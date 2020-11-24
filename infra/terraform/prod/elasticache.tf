resource "aws_elasticache_cluster" "wikijump_cache" {
  cluster_id           = "wikijump-cache-${var.environment}"
  engine               = "memcached"
  node_type            = var.cache_ec2_size
  num_cache_nodes      = var.cache_num_nodes
  parameter_group_name = "default.memcached1.6"
  port                 = 11211
  subnet_group_name    = aws_elasticache_subnet_group.cache_subnet.name
  security_group_ids   = [aws_security_group.elasticache_sg.id]
}

resource "aws_elasticache_subnet_group" "cache_subnet" {
  name       = "wikijump-${var.environment}-cache-subnet"
  subnet_ids = [aws_subnet.cache_subnet.id]
}

resource "aws_security_group" "elasticache_sg" {
    name            = "elasticache_sg_${var.environment}"
    description     = "Allow 11211 inbound"

    ingress {
        description = "Memcached"
        from_port   = 11211
        to_port     = 11211
        protocol    = "tcp"
        cidr_blocks = [var.container_subnet]  # Probably a cleaner way to do this is getting vars from ECS
    }

    egress {
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }
    vpc_id = aws_vpc.wikijump_vpc.id
}

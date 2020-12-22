# VPC

resource "aws_vpc" "wikijump_vpc" {
  cidr_block                       = var.vpc_cidr_block
  enable_dns_hostnames             = true
  enable_dns_support               = true
  assign_generated_ipv6_cidr_block = true
}

# Subnets

resource "aws_subnet" "elb_subnet" {
  vpc_id                  = aws_vpc.wikijump_vpc.id
  cidr_block              = var.elb_subnet
  map_public_ip_on_launch = true
  availability_zone       = format("%s%s", var.region, var.availability_zone)
  depends_on              = [aws_internet_gateway.wikijump_igw]
}

resource "aws_subnet" "container_subnet" {
  vpc_id                  = aws_vpc.wikijump_vpc.id
  cidr_block              = var.container_subnet
  map_public_ip_on_launch = true
  availability_zone       = format("%s%s", var.region, var.availability_zone)
  depends_on              = [aws_internet_gateway.wikijump_igw]
}

# Gateways

resource "aws_internet_gateway" "wikijump_igw" {
  vpc_id = aws_vpc.wikijump_vpc.id
}

# Routes

resource "aws_route_table" "public_route" {
  vpc_id = aws_vpc.wikijump_vpc.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.wikijump_igw.id
  }
}

resource "aws_route_table_association" "pub_elb" {
  subnet_id      = aws_subnet.elb_subnet.id
  route_table_id = aws_route_table.public_route.id
}

resource "aws_route_table_association" "pub_container" {
  subnet_id      = aws_subnet.container_subnet.id
  route_table_id = aws_route_table.public_route.id
}

# Elastic IPs

resource "aws_eip" "elb_eip" {
  vpc        = true
  depends_on = [aws_internet_gateway.wikijump_igw]
}

# Security Groups

resource "aws_security_group" "ecs_nodes" {
  name   = "ECS nodes for Wikijump dev"
  vpc_id = aws_vpc.wikijump_vpc.id
}

resource "aws_security_group_rule" "ping" {
  from_port         = 8081
  to_port           = 8081
  protocol          = "tcp"
  cidr_blocks       = [var.elb_subnet]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "ingress"
}

resource "aws_security_group_rule" "http" {
  from_port         = 80
  to_port           = 80
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "ingress"
}

resource "aws_security_group_rule" "https" {
  from_port         = 443
  to_port           = 443
  protocol          = "tcp"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "ingress"
}

resource "aws_security_group_rule" "egress" {
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  cidr_blocks       = ["0.0.0.0/0"]
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "egress"
}

resource "aws_security_group_rule" "ingress_self" {
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  self              = true
  security_group_id = aws_security_group.ecs_nodes.id
  type              = "ingress"
}

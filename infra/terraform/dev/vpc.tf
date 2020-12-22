# VPC

resource "aws_vpc" "wikijump_vpc" {
  cidr_block                       = var.vpc_cidr_block
  enable_dns_hostnames             = true
  assign_generated_ipv6_cidr_block = true
}

# Subnets

resource "aws_subnet" "elb_subnet" {
  vpc_id                  = aws_vpc.wikijump_vpc.id
  cidr_block              = var.elb_subnet
  map_public_ip_on_launch = true

  depends_on = [aws_internet_gateway.wikijump_igw]
}

resource "aws_subnet" "container_subnet" {
  vpc_id                  = aws_vpc.wikijump_vpc.id
  cidr_block              = var.container_subnet
  map_public_ip_on_launch = true

  depends_on = [aws_internet_gateway.wikijump_igw]
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

resourse "aws_route_table_association" "pub_elb" {
  subnet_id      = aws_subnet.elb_subnet.id
  route_table_id = aws_route_table.public_route.id
}

resourse "aws_route_table_association" "pub_container" {
  subnet_id      = aws_subnet.container_subnet.id
  route_table_id = aws_route_table.public_route.id
}

# Elastic IPs

resource "aws_eip" "elb_eip" {
  vpc        = true
  depends_on = [aws_internet_gateway.wikijump_igw]
}

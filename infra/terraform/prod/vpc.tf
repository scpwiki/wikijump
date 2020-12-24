# VPC

resource "aws_vpc" "wikijump_vpc" {
    cidr_block                          = var.vpc_cidr_block
    enable_dns_hostnames                = true
    assign_generated_ipv6_cidr_block    = true
}

# Subnets

resource "aws_subnet" "elb_subnet" {
    vpc_id          = aws_vpc.wikijump_vpc.id
    cidr_block              = var.elb_subnet
    map_public_ip_on_launch = true

    depends_on = [aws_internet_gateway.wikijump_igw]
}

resource "aws_subnet" "container_subnet" {
    vpc_id  = aws_vpc.wikijump_vpc.id
    cidr_block      = var.container_subnet
}

resource "aws_subnet" "database_subnet_a" {
    vpc_id  = aws_vpc.wikijump_vpc.id
    cidr_block      = var.database_subnet_a
    availability_zone = "${var.region}a"
}

resource "aws_subnet" "database_subnet_b" {
    vpc_id  = aws_vpc.wikijump_vpc.id
    cidr_block      = var.database_subnet_b
    availability_zone = "${var.region}b"
}

resource "aws_subnet" "cache_subnet" {
    vpc_id  = aws_vpc.wikijump_vpc.id
    cidr_block      = var.cache_subnet
}

# Gateways

resource "aws_internet_gateway" "wikijump_igw" {
  vpc_id = aws_vpc.wikijump_vpc.id
}

# Routes



# Elastic IPs

resource "aws_eip" "elb_eip" {
    vpc         = true
    depends_on  = [aws_internet_gateway.wikijump_igw]
}

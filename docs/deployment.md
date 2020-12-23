# Remote Deployment

This document will cover the process of deploying Wikijump on Amazon Web Services.

## General Architecture

Communication with Wikijump begins at DNS. If the address is for a wikijump.com subdomain, or a custom domain CNAMEd to wikijump.com, it will hit an Elastic Load Balancer. Specifically, it will resolve to the static (elastic) IP address of a Network Load Balancer listening on ports 80 and 443, but not terminating any SSL connections. It will proxy all this traffik to the Traefik edge proxy containers running in Elastic Container Service. Traefik handles the business of SSL termination and the permanent redirecting of http to https. It will then act as a reverse proxy, forwarding the request on to other containers running php-fpm and nginx. These containers are connected to a caching layer (memcached) and a database (postgres).

If the address is for a wjfiles.com subdomain, it will instead hit a CloudFront distribution. This will terminate SSL and examine the requested path. If it is for a file asset, it will retrieve the file from S3. If it is for a code-type asset, it will proxy the request to Traefik, which goes to the same php-fpm and nginx containers to return the code. As objects retrieved in this way should generally be static assets, we will make use of caching to reduce the load on our internals.

## Prerequisites

Deployment was designed to need a minimal amount of work done in advance, but there is always some. Feel free to contribute code to run `aws` CLI calls for some of this.

1. You will need [Terraform](https://www.terraform.io) as well as a place to store Terraform state files. We use Terraform Cloud which is free for teams of up to 5 users, but you can also do things like storing the state files in S3.
2. You will need to make an IAM user for Terraform to use to create and update everything. A JSON file for the IAM Policy is forthcoming.
3. You will need to make an IAM user for your CI/CD (GitHub Actions for us) to use to push Docker images. A JSON file for the IAM policy is forthcoming.


## Instructions

1. Check the provided tfvars file and replace items as necessary. You also need to check the locals in the `infra/terraform/(environment)/init/init.tf` file and make sure you're okay with the provided region and environment name.
2. Run `terraform apply` on the `infra/terraform/(environment)/init` folder. This will create the ECR repositories and store their URLs in Parameter Store.
3. Push your initial docker images to ECR.
4. Run `terraform apply` on the `infra/terraform/(environment)/deploy` folder. This will do everything else to set up the environment.

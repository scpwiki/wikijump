# Remote Deployment

This document will cover the process of deploying Wikijump on Amazon Web Services.

## General Architecture

Communication with Wikijump begins at DNS. If the address is for a `wikijump.com` subdomain, or a custom domain `CNAME`d to `wikijump.com`, it will hit an Network Load Balancer with an Elastic IP reservation listening on ports 80 and 443, but not terminating any SSL connections. It will proxy all this traffic to the Traefik edge proxy containers running in Elastic Container Service. Traefik handles the business of SSL termination and the permanent redirecting of HTTP to HTTPS. It will then act as a reverse proxy, forwarding the request on to other containers running `php-fpm` and `nginx`. These containers are connected to a caching layer (`memcached`) and a database (`postgres`).

If the address is for a `wjfiles.com` subdomain, it will instead hit a CloudFront distribution. This will terminate SSL and examine the requested path. If it is for a file asset (`local--files/*`), it will retrieve the file from S3. If it is for a code-type asset (`local--code/*`), it will proxy the request to Traefik, which goes to the same `php-fpm` and `nginx` containers to return the code. As objects retrieved in this way should generally be static assets, we will make use of caching to reduce the load on our internals.

## Prerequisites

Deployment was designed to need a minimum of work done outside the scope of this package. All that should be required is storing some AWS credentials to the CI/CD provider of your choosing.

*Note: Deploying the software via the docker and terraform packages is the only supported configuration. You can certainly deploy another way if you're able, but we likely won't be able to help you troubleshoot it. In particular, modifying the deployment around SSL configuration is highly discouraged, and we cannot offer support on insecure deployments.*

1. You will need [Terraform](https://www.terraform.io) as well as a place to store Terraform state files. We use Terraform Cloud which is free for teams of up to 5 users, but you can also do things like storing the state files in S3.
2. You will need to make an IAM user for Terraform to use to create and update everything. A JSON file for the IAM Policy is forthcoming.
3. You will need to make an IAM user for your CI/CD (GitHub Actions for us) to use to push Docker images. A JSON file for the IAM policy is forthcoming.


## Instructions

1. Check the provided tfvars file and replace items as necessary. You also need to check the locals in the `infra/terraform/(environment)/init/init.tf` file and make sure you're okay with the provided region and environment name.
2. Run `terraform apply` on the `infra/terraform/(environment)/init` folder. This will create the ECR repositories and store their URLs in Parameter Store.
3. Push your initial docker images to ECR.
4. Run `terraform apply` on the `infra/terraform/(environment)/deploy` folder. This will do everything else to set up the environment.

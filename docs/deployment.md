# Remote Deployment

This document will cover the process of deploying Wikijump on Amazon Web Services.

## General Architecture

Communication with Wikijump begins at DNS. If the address is for a wikijump.com subdomain, or a custom domain CNAMEd to wikijump.com, it will hit an Elastic Load Balancer. Specifically, it will resolve to the static (elastic) IP address of a Network Load Balancer listening on ports 80 and 443, but not terminating any SSL connections. It will proxy all this traffik to the Traefik edge proxy containers running in Elastic Container Service. Traefik handles the business of SSL termination and the permanent redirecting of http to https. It will then act as a reverse proxy, forwarding the request on to other containers running php-fpm and nginx. These containers are connected to a caching layer (memcached) and a database (postgres).

If the address is for a wjfiles.com subdomain, it will instead hit a CloudFront distribution. This will terminate SSL and examine the requested path. If it is for a file asset, it will retrieve the file from S3. If it is for a code-type asset, it will proxy the request to Traefik, which goes to the same php-fpm and nginx containers to return the code. As objects retrieved in this way should generally be static assets, we will make use of caching to reduce the load on our internals.

## Prerequisites

Deployment was designed to need a minimal amount of work done in advance, but there is always some. Feel free to contribute code to run `aws` CLI calls for some of this.

1. You will need [Terraform](https://www.terraform.io) as well as a place to store Terraform state files. We use Terraform Cloud which is free for teams of up to 5 users, but you can also do things like storing the state files in S3.
2. You will need to make an IAM user for Terraform to use to create and update everything. A JSON file for the IAM Policy is forthcoming.
3. You will need to make an IAM user for your CI/CD (GitHub Actions for us) to use to push Docker iamges. A JSON file for the IAM policy is forthcoming.
4. You will need to create several Elastic Container Registry Repositories that your images will go in. This is to solve a chicken-and-egg problem where GitHub Actions needs to push updated images to a repository that doesn't exist yet. I would suggest enabling KMS Encryption and Scan On Push, and disabling Tag Immutability on all repositories. One registry is plenty, and it will be expected to have `wikijump/traefik`, `wikijump/memcached`, `wikijump/postgres`, and `wikijump/php-fpm` as repositories.

## Instructions

A couple of things need to happen in order for things to build correctly.
1. Run the terraform deployment. This will generate most of the infrastructure for you not counting the container piece.
2. You need to export some ARNs to your CI/CD tool: 

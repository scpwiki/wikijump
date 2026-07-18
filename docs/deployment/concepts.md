## Deployment Concepts

This document contains information about our deployment environments which are common between both dev and prod.

For the continuous local browser-facing runtime, follow the [runtime drift policy](runtime-drift-policy.md). It defines the single merged-head owner of port 443, candidate isolation, promotion evidence, and canaries.

We are using [Komodo](https://komo.do)-based dev tier hosting. This self-hostable solution offers a web UI with per-user permissions for monitoring and maintaining a cluster, and features infrastructure-as-code. This way we can, to the extent reasonably possible, avoid machines-as-pets. Provided that our databases remain intact / can be restored from backup, then in principle all our infrastructure should be recreatable from the source code.

As such, we are documenting setup steps and we store infrastructure files in `install/{dev,prod}/komodo/`.

Komodo has the concept of a **core**, which is the leader of a cluster, responsible for overseeing the other machines and maintaining the health of deployed services and other resources.

It can support one or more **machines**, which run Periphery, management software which is responsible for effectuating the instructions given to it by the Komodo Core instance.

On dev, we only have one VPS, which serves as both the core and its only machine. Additionally, we will not have a dedicated database here, instead using a disposable container which is recreated on dev tier deployment.

On prod, we will have multiple machines. One of them will serve as the core, and the others will be worker machines.

The machine the core is installed on is always `Local`. Any other machines are numbered `Machine 1`, `Machine 2`, etc. (Numbering begins from one since the core is implicitly machine zero)

Our host here is DigitalOcean, due to its reliability, good service offering and generous bandwidth allowance, but the instructions here should apply to nearly any cloud provider.

Our infrastructure uses some particular vendors; this section explains quirks associated with them (and what would be needed to replace it with another vendor):

* **Using DigitalOcean for compute and other services.** There are a number of cloud providers which offer VPSes very similar to DigitalOcean's offerings.
* **Using DigitalOcean for DNS management.** ACME challenges normally rely on setting up a temporary web server, but the DNS ACME challenge (required for wildcard certificates) requires setting a TXT DNS records. Thus, we need a restricted DigitalOcean token that gives access only to DNS settings. There are a [large number of Caddy providers for this kind of ACME challenge](https://caddy.community/t/how-to-use-dns-provider-modules-in-caddy-2/8148).
* **Using AWS ECR as a container registry.** This is primarily for cost, as other registries (e.g. DigitalOcean) are a bit expensive for how much we use them. However, any container registry is usable, and would in fact be less complicated since you can just use the default Komodo Periphery image without the `amazon-ecr-credential-helper` addition.
* **Using GitHub as code forge.**  The Wikijump codebase is a standard git repository, and Komodo has good support for other git forges.
* **Using GitHub for CI/CD.** The primary lock-in here comes from use of GitHub Actions for CI/CD. Additionally, GitHub Container Registry is used for Minio (only used for integration testing in GitHub Actions).

## Dev Environment

See the [deployment concepts document](concepts.md) for an introduction to our deployment strategy. This document describes hosting the dev tier, that is, `wikijump.dev`.

1. Create a Virtual Private Server with Ubuntu 24.04 LTS.
2. Set up a non-root administrator account:
```
# adduser --disabled-password maintainer
# gpasswd -a maintainer sudo
# passwd -d maintainer
```
3. Add SSH keys to enable login as `maintainer`:
```
# su maintainer
$ cd
$ mkdir -m700 .ssh
$ nano .ssh/authorized_keys
$ chmod 600 .ssh/authorized_keys
```
Then, ensure you can SSH to the machine as `maintainer`. The remaining instructions assume you are logged in as `maintainer`, not `root`.
4. Disable password-based SSH (if not already disabled):
```
$ sudoedit /etc/ssh/sshd_config
PasswordAuthentication no
PermitEmptyPasswords no
$ sudo systemctl reload ssh.service
```
5. Install Docker and other dependencies:
```
$ sudo apt install docker.io docker-compose-v2 docker-buildx amazon-ecr-credential-helper
```
6. (For AWS ECR) Set up the ECR credential helper:
```
$ sudo mkdir -m 700 ~root/.docker
$ sudoedit ~root/.docker/
{
	"credHelpers": {
		"public.ecr.aws": "ecr-login",
		"575596218155.dkr.ecr.us-east-2.amazonaws.com": "ecr-login"
	}
}
```
7. Before starting Komodo, enable a host firewall or security-group policy. The permanent public inbound ports should be 22 (for SSH), and 80 and 443 (for HTTP traffic). During bootstrap, allow Komodo's port 9120 only from a trusted administrator source. The `database`, `cache`, `deepwell`, `framerail`, and `wws` services are internal to the Compose network and must never be published directly to public host interfaces.
8. Install Komodo:
When multiple servers are initiated for the same tier, note that *only one machine should have a Komodo Core*. All the servers need a Periphery instance to be able to talk to the one machine running Komodo Core.

The files to use here are located in the current directory, and for `compose.env` see `compose.env.example` to populate the missing fields.
```
$ sudo mkdir -m 700 -p /var/lib/komodo/backups
$ mkdir ~/komodo
$ cd ~/komodo
Copy docker-compose.yaml from install/dev/komodo/docker-compose.yaml
Create compose.env based on install/dev/komodo/compose.env.example
$ sudo docker compose -p komodo -f docker-compose.yaml --env-file compose.env up -d
Ensure that it's running as expected:
$ sudo docker compose -p komodo -f docker-compose.yaml --env-file compose.env ps
```
9. Log in to Komodo.
Using the admin password you generated for `compose.env`, log in to Komodo via `http://[IP ADDRESS]:9120/`.
10. Bootstrap resource sync.
In order to add the rest of the infrastructure, we need to add a git repository and a resource sync. Then, Komodo can use the `*.toml` files in `install/dev/komodo/` to set up the rest of the infrastructure.

  1. Go to **repos**. See `install/dev/komodo/sources.toml` and add the fields as appropriate.
  2. Go to **resource syncs**. See `install/dev/komodo/resource-sync.toml` add add the fields as appropriate.
  3. As a **one-time change**, set "Sync Variables" to true.
  4. Click the "refresh" button, verify that proposed infrastructure changes look good, then apply.
  5. Set "Sync Variables" back to false.
11. Add secrets.
It is not good practice to add secrets to code, and triply so if the repository is public. As such, `install/dev/komodo/variables.toml` is missing values for those marked "secret" (see the file for more information). Some of these are secret values that need to be generated, and some come from your infrastructure. Fill in the values as appropriate.
12. Go to the `wikijump-dev` stack and **pull images**. If everything is configured properly so far, it should be able to retrieve the images and be in a state to deploy it. Keep the temporary port 9120 rule restricted to the trusted administrator source throughout bootstrap.
13. Now, deploy the `wikijump-dev` stack. This will first build the local images (the two databases) and attempt to start the containers per the topology in `docker-compose.yaml`.
On the first deploy, it may take some time to populate the database. You may need to restart services dependent on `deepwell` (i.e. `caddy`, `framerail`, `wws`) if they are reporting as unhealthy.
14. Confirm that Caddy serves the Komodo UI at `https://deploy.wikijump.dev`, then remove the temporary inbound rule for port 9120. Komodo should remain reachable only through Caddy after bootstrap.

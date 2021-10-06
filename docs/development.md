# Local Development

This document will explain how to set up Wikijump on your machine for local development.

## Environment

### Operating System

To run Wikijump, you will need to be running Linux.

If you are running Windows, you have two options:

* Install a Virtual Machine (VM) running Linux, for example using [VirtualBox](https://www.virtualbox.org/), and work from inside it as if it were Linux
* (Recommended) Install [Windows Subsystem for Linux 2 (WSL2)](https://docs.microsoft.com/en-us/windows/wsl/install-win10)

In either case, when choosing a Linux distribution, Ubuntu is fine.

## Installation

The `install` folder has everything you need to run a local Wikijump install either in a container or on metal or a VM.

### Installation via [Docker](https://www.docker.com/) locally

The recommended way to install Wikijump is via Docker. Docker is a container system that lets you easily create and destroy different Wikijump builds, and it also acts like a sandbox to protect the rest of your system from dependency pollution.

You will need Docker installed and running:

<table>
  <thead><tr><th>Ubuntu / Ubuntu VM</th><th>Windows via WSL2</th></tr></thead>
  <tbody valign="top"><tr>
    <td>
      <p><pre># apt install docker</pre></p>
      <p>Start the Docker daemon in another terminal, and leave it running in the background:</p>
      <p><pre>$ dockerd</pre></p>
    </td>
    <td>
      <p><pre># apt install docker</pre></p>
      <p>Install <a href="https://docs.docker.com/docker-for-windows/install-windows-home">Docker Desktop</a>, which you'll be using as the Docker daemon, and leave it running in the background.</p>
      <p>If you have set <code>appendWindowsPath=false</code> in your WSL config, then you may hit an error along the lines of <code>"docker-credential-desktop.exe": executable file not found in $PATH</code>. In this case you should either add <code>/mnt/c/Program\ Files/Docker/Docker/resources/bin</code> to your PATH, or <a href="https://github.com/rossjrw/dotfiles/blob/3c5445abb138b735cc3caf61f070c9125fa87d2f/.profile#L28">create a symlink</a> in a directory that is in your PATH already.</p>
    </td>
  </tr></tbody>
</table>

Then install [Docker Compose](https://docs.docker.com/compose/).

You will need to provide a `docker-compose.yaml` file to run Wikijump. The domains to be used, TLS certificates, and volumes will be set here. See `docker-compose.yaml.example` for what this file may look like. Note that for a local deployment, dummy domains like `wikijump.localhost` will work fine.

Additionally you will need to generate TLS certificates. Fill in the domain variables with the same values you used in your `docker-compose.yaml` file.

```sh
$ openssl req \
	-x509 \
	-newkey rsa:4096 \
	-sha256 \
	-days 3650 \
	-nodes \
	-keyout cert.key \
	-out cert.crt \
	-subj "/CN=${MAIN_DOMAIN}" \
	-addext "subjectAltName=DNS:${FILES_DOMAIN},DNS:*.${MAIN_DOMAIN},DNS:*.${FILES_DOMAIN}"
```

Once those are configured, you can use the provided `docker-compose.yaml` file to get the containers started. The following will build all the images, and then run new containers with the prefix `wikijump`:

```
$ cd install/local/dev
$ docker-compose -p wikijump up
```

When running, navigate to http://www.wikijump.localhost/ in your browser. This will contain the containerized Wikijump installation.
Your browser will probably complain that the site is insecure, citing that the page has a self-signed certificate. During local development, this is unfortunate but expected.

-----

If you want to stop the containers, use the following:

```
$ docker-compose -p wikijump stop
```

(Or interrupt the running `docker-compose` process with Ctrl+C)

If you want to delete the containers, you can completely bring down the deployment:

```
$ docker-compose -p wikijump down
```

It's useful to keep track of existing Docker images and containers, and destroy them when you no longer need them, so you don't waste space rebuilding the same image over and over. If you are using Docker Desktop, you can manage containers and images from the GUI. Otherwise, on command line:

```
$ docker container ls  # List containers
$ docker rm [ID]       # Destroy the container with this ID
$ docker images        # List images
$ docker rmi [ID]      # Remove the image with this ID
```

Note: If you enable two-factor authentication on a local container you may find that
the clock drift is too great for TOTP codes to work. In docker-compose (`wsl -d docker-compose`) 
you can enter this command to sync your time up:

```
$ ntpd -d -q -n -p 0.pool.ntp.org
```

## Entering the container

If you want to enter the container to make temporary changes, you can do so by entering it with a CLI. From Docker Desktop, after running `docker-compose up`, find the Wikijump app and within it the `php-fpm` container, then click the 'CLI' button. Or from the command line:

```
$ docker exec -it [name of container] sh
```

...where `[name of container]` is the name of the PHP-FPM container from `docker container ls`.

One reason you may need to enter the container is to adjust the Wikijump config. For example, if you use a port other than 80 for your Docker container, you will need to edit `site.custom_domain` to add the port number (e.g. "`www.wikijump.localhost:8080`"). Alternatively, use curl to set the domain directly (e.g. "`-H 'www.wikijump.localhost'`")

After editing the Wikijump config, you may need to restart nginx inside the container:

```
# service nginx restart
```

## Seeing local changes

If you want changes made to your copy of Wikijump from outside Docker to be visible inside Docker, you can do so by creating a [bind mount](https://docs.docker.com/storage/bind-mounts/), which temporarily overwrites files inside the container with files from outside the container.

There is a development-oriented Docker Compose config file created with this in mind, which should be used as a config override:

```
$ docker-compose -p wikijump -f docker-compose.yaml -f docker-compose.dev.yaml up
```

This will map the directories specified in `docker-compose.dev.yaml` into the container, so you can see your changes live. You should edit this file as you need, but do not commit any personal changes to it.

Be sure that you have produced the javascript bundle before attempting to visit your local instance in a web browser:
```
$ cd web
$ npm install
$ npm run build
```

Some changes (e.g. to `lib/`) require that the container be rebuilt:
```
$ docker-compose -p wikijump -f docker-compose.yaml -f docker-compose.dev.yaml build
```

## Recommended Reading

Here is a list of some shorter resources that may be helpful if you wish to contribute to Wikijump's source:

* [Blade templating documentation](https://laravel.com/docs/8.x/blade)

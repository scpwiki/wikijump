# Local Frontend Development

This document will help you get started with frontend oriented development for Wikijump.

## Overview

Wikijump, for local development anyways, runs in a set of Docker containers. Docker is a way of containerizing, or in the case of Windows, also virtualizing, Linux images.

> ### For Windows:
>
> For Windows, you will be using WSL2, a way of running a Linux distribution simultaneously with Windows. You will need Windows 10 for this. The only alternative to WSL2 would be using a Linux virtual machine - but using a VM isn't recommended and it won't be explained how to use one here.
>
> It is recommended that you use Ubuntu for the WSL2 distribution. Ubuntu in particular has considerations made for WSL2 use and in general will be the most reliable way forward.
>
> [WSL2 download and installation page](https://docs.microsoft.com/en-us/windows/wsl/install-win10)

In general, the process of running Wikijump on your machine to undertake development work is using what is called the "local" environment. The other environments are "development", which is a build running on a server for the sake of development work, and "production", which is the environment that consumers actually connect to.

## Setup: Utilites and Programs

You will need some utilites and programs to get started. You will to install the following:

- [PHP 7.4](https://www.php.net/downloads)
- [Composer (PHP dependency manager)](https://getcomposer.org/)
- [Node (and NPM, which comes with it), v15 or greater](https://nodejs.org/en/)
- [PNPM v6](https://pnpm.io/installation)
- [Docker](https://www.docker.com/get-started)

> ### Docker on Linux:
>
> On Linux, you install Docker like normal:
>
> ```sh
> $ apt install docker
> ```
>
> Then you just leave the daemon running in the background:
>
> ```sh
> $ dockerd
> ```

> ### Docker on Windows:
>
> On Windows, you will need to install Docker both on Windows directly and inside your WSL2 distribution.
>
> Windows is relatively simple - just install Docker Desktop. For WSL2, you will need to run the following command in the WSL2 shell:
>
> ```sh
> $ apt install docker
> ```
>
> Once that's all done, just leave Docker Desktop running in the background whenever you want to do development work.

Node, NPM, and PNPM are well-behaved on Windows and Linux, and the difference in usage between operating systems is negligable.

## Setup: Configuration

Wikijump has a lot of configuration options that can set. The primary configuration file can be found in `web/conf/`, named `wikijump.ini`. Freshly cloned, your Wikijump repo won't have this file. However, it will have an example configuration, `wikijump.ini.example`, which you can just copy and rename to `wikijump.ini` for the time being.

Second, there is a Docker configuration file that configures the various containers that host Wikijump in the local environment. You can find this file in `install/local/dev/`, named `docker-compose.yaml`. Like before, you won't have this file with a fresh Wikijump clone. However, again like before, there is a `docker-compose.yaml.example`, which you can just copy and rename to `docker-compose.yaml`.

## Setup: Hosting

You will need to configure your HOSTS file and create a TLS certificate. This is for configuring a connection to `https://www.wikijump.test`. Note that the certificate you will be creating will be "self-signed", meaning that your browser will complain about it. Ignoring the warnings is fine.

On Windows, your HOSTS file can be found in `C:\Windows\System32\drivers\etc\hosts`. On Linux, it's simply `/etc/hosts`. Your editing program on Windows will need to be ran as administrator.

Add the following lines to your HOSTS file:

```
127.0.0.1          wikijump.test               Wikijump
127.0.0.1          www.wikijump.test           Wikijump
127.0.0.1          wjfiles.test                Wikijump
127.0.0.1          profiles.wikijump.test      Wikijump
127.0.0.1          template-en.wikijump.test   Wikijump
127.0.0.1          sandbox.wikijump.test       Wikijump
127.0.0.1          scp-wiki.wikijump.test      Wikijump
::1                wikijump.test               Wikijump
::1                www.wikijump.test           Wikijump
::1                wjfiles.test                Wikijump
::1                profiles.wikijump.test      Wikijump
::1                template-en.wikijump.test   Wikijump
::1                sandbox.wikijump.test       Wikijump
::1                scp-wiki.wikijump.test      Wikijump
```

To create the TLS certificate, you will need to run a creation command in the `install/local/dev/ssl` folder. This command needs to be ran in a Linux shell - so run this in a WSL2 terminal on Windows.

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

Replace `${MAIN_DOMAIN}` and `${FILES_DOMAIN}` with the relevant domain names. These can be found in the previously described `docker-compose.yaml` file. If you're just using the example file, these names will be `wikijump.test` and `wjfiles.test` respectively.

> You may need `openssl` if you don't have it:
>
> ```
> $ apt install openssl
> ```

## Setup: Dependencies

Believe me - you're almost done.

You will need to install Wikijump's NPM and Composer dependencies. You should do PHP/Composer first.

Run the following command in `web/`:

```sh
$ composer install
```

> ### For Windows:
>
> On Windows, Composer might be troublesome. It might be easier to run it in WSL2 instead - make sure to install PHP7.4 and Composer on your WSL2 install to do this.

After Composer is done, which may take a while, you can then install the NPM dependencies using PNPM.

In both `client/`, and `web/`, in that order, run the following:

```sh
$ pnpm install
```

## Building

You can now finally build the JS/TS codebase and Docker images.

Run the following command in `web/`:

```sh
$ pnpm build
```

This will first build the JS/TS codebase, which probably won't take very long at all, and then will build the Docker containers. _This might take a surprisingly long time_. Thankfully, Docker's build step is _heavily_ cached. It won't take this long again.

## Development

To run the development mode, run the following in `web/`:

```sh
$ pnpm dev
```

This will run everything in a watch-mode, and start the Docker containers. Once it's started, you can connect to `https://wikijump.test` and make changes. Changes should be reflected automatically - your local machine's filesystem has been "bound" to the Docker containers' filesystem. Changes will propagate (FYI: one-way only) to the containers.

You can just kill the terminal (`CTRL + C` usually) when you want to stop the server. If you're doing a restart - as in stopping and then immediately restarting the development mode - you should probably wait a moment before actually restarting. The Docker containers don't shutdown immediately and they will do so in the background.

If for some reason the containers aren't shutting down, you can run the following:

```sh
$ pnpm compose stop
```

If you want to entirely _reset_ the containers, as their data is otherwise persistent even across restarts, you can run the following:

```sh
$ pnpm compose down
```

You can also manually bring them up without the JS/TS development mode using `pnpm compose up`, but there isn't much point to this for frontend work.

## Relevant Documentation

- [Laravel](https://laravel.com/docs/8.x/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Blade templates](https://laravel.com/docs/8.x/blade)
- [PNPM](https://pnpm.io/)
- [Vite](https://vitejs.dev/)

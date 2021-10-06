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

Notice that in `docker-compose.yaml`, there are configuration options for the domains to use. For development purposes, these are set to `wikijump.localhost`. This is the domain you will be connecting to, e.g. `http://www.wikijump.localhost`. The TLD `.localhost` is just like the usual `localhost` domain. Your browser trusts both of these domains implicitly so there is no need to use `HTTPS`.

Finally, you will need a `.env` file in the `web/` root. There is a `.env.example` file that you can copy and rename.

## Setup: Dependencies

You will need to install Wikijump's NPM and Composer dependencies. You should do PHP/Composer first.

Run the following command in `web/`:

```sh
$ composer install
```

> ### For Windows:
>
> On Windows, Composer might be troublesome. It might be easier to run it in WSL2 instead - make sure to install PHP7.4 and Composer on your WSL2 install to do this.

After Composer is done, which may take a while, you can then install the NPM dependencies using PNPM.

In `web/` run the following:

```sh
$ pnpm install
```

## Building

You can now finally build the JS/TS codebase and Docker images.

You can run the following command in `web/` for building:

```sh
$ pnpm build
```

This will first build the JS/TS codebase, which probably won't take very long at all, and then will build the Docker containers. _This might take a surprisingly long time_. Thankfully, Docker's build step is _heavily_ cached. It won't take this long again.

## Development

To run the development mode, run the following in `web/`:

```sh
$ pnpm dev
```

This command will run everything in watch-mode, and start the Docker containers. However, it will first ask you if you want to build the containers, as in what `pnpm build` does. This is asked because building may take a very long time.

Once everything has started, you can connect to `http://www.wikijump.localhost`. Changes you make to the codebase should automatically be applied to the containers, as your machine's filesystem has been "bound" to the containers' filesystem. This is one-way, so a container can't modify your filesystem.

You can just kill the terminal (`CTRL + C` usually) when you want to stop the server. If you're on Windows, you may want to _hold_ `CTRL + C` to bypass annoying `Terminate batch job?` prompts, and if you do so, make sure to wait for the Docker containers to exit before restarting the development mode.

If for some reason the containers aren't shutting down, you can run the following:

```sh
$ pnpm compose stop
```

If you want to entirely _reset_ the containers, as their data is otherwise persistent even across restarts, you can run the following:

```sh
$ pnpm compose down
```

You can also manually bring them up without the JS/TS development mode using `pnpm compose up`, but there isn't much point to this for frontend work.

## Using `sudo`

If your Linux OS doesn't agree with the normal `pnpm dev`, `pnpm build`, and `pnpm compose`, then you can use the `-sudo` variants of these commands, e.g. `pnpm dev-sudo`.

## Relevant Documentation

- [Laravel](https://laravel.com/docs/8.x/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Blade templates](https://laravel.com/docs/8.x/blade)
- [PNPM](https://pnpm.io/)
- [Vite](https://vitejs.dev/)

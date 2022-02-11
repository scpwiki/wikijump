# Local Development

This document will explain how to set up Wikijump on your machine for local development.

## Deployment

The `install` folder has everything you need to run a local Wikijump install either in a container or on metal or a VM.

The recommended way to install Wikijump is via Docker, utilizing pnpm. Docker is a way of containerizing, or in the case of Windows or Mac, also virtualizing, Linux images. It lets you easily create and destroy different Wikijump builds, and it also acts like a sandbox to protect the rest of your system from dependency pollution. PNPM is an alternate package manager for NodeJS.

> ### For Windows:
>
> For Windows, you will need WSL2, a way of running a Linux distribution simultaneously with Windows. You will need Windows 10 for this. The only alternative to WSL2 would be using a Linux virtual machine such as [VirtualBox](https://www.virtualbox.org/) - but using a VM isn't recommended and it won't be explained how to use one here.
>
> It is recommended that you use Ubuntu for the WSL2 distribution. Ubuntu in particular has considerations made for WSL2 use and in general will be the most reliable way forward.
>
> [WSL2 download and installation page](https://docs.microsoft.com/en-us/windows/wsl/install-win10)

You will need [Docker](https://www.docker.com/) installed and running:

<table>
  <thead><tr><th>Ubuntu / Ubuntu VM</th><th>Windows via WSL2</th></tr></thead>
  <tbody valign="top"><tr>
    <td>
      <p><pre># apt install docker.io</pre></p>
      <p>Start the Docker daemon via systemd or whatever your service manager is.</p>
      <p><pre>$ sudo systemctl enable --now docker.service</pre></p>
    </td>
    <td>
      <p>Install <a href="https://docs.docker.com/docker-for-windows/install-windows-home">Docker Desktop</a>, which you'll be using as the Docker daemon, and leave it running in the background.</p>
      <p>If you have set <code>appendWindowsPath=false</code> in your WSL config, then you may hit an error along the lines of <code>"docker-credential-desktop.exe": executable file not found in $PATH</code>. In this case you should either add <code>/mnt/c/Program\ Files/Docker/Docker/resources/bin</code> to your PATH, or <a href="https://github.com/rossjrw/dotfiles/blob/3c5445abb138b735cc3caf61f070c9125fa87d2f/.profile#L28">create a symlink</a> in a directory that is in your PATH already.</p>
    </td>
  </tr></tbody>
</table>

## Setup: Utilites and Programs

You will need some utilites and programs to get started. You will to install the following:

- [Docker](https://www.docker.com/get-started) (see above)
- [Docker Compose](https://docs.docker.com/compose/)
- [NodeJS (and NPM, which comes with it), v15 or greater](https://nodejs.org/en/)
- [PNPM v6](https://pnpm.io/installation)

Node, NPM, and PNPM are well-behaved on Windows and Linux, and the difference in usage between operating systems is negligable.

## Setup: Configuration

Wikijump has a lot of configuration options that can set. The primary configuration file can be found in `web/conf/`, named `wikijump.ini`. Freshly cloned, your Wikijump repo won't have this file. However, it will have an example configuration, `wikijump.ini.example`, which you can just copy and rename to `wikijump.ini` for the time being.

Second, there is a Docker configuration file that configures the various containers that host Wikijump in the local environment. You can find this file in `install/local/dev/`, named `docker-compose.yaml`.

Notice that in `docker-compose.yaml`, there are configuration options for the domains to use. For development purposes, these are set to `wikijump.localhost`. This is the domain you will be connecting to, e.g. `http://www.wikijump.localhost`. The TLD `.localhost` is just like the usual `localhost` domain. Your browser trusts both of these domains implicitly so there is no need to use `HTTPS`.

Finally, you will need a `.env` file in the `web/` root. There is a `.env.example` file that you can copy and rename.

## Setup: Dependencies

You will need to install Wikijump's NPM dependencies. Navigate to the `web/` directory and run the following:

```sh
$ pnpm install
```

## Building

You can now finally build the Docker images using the following command:

```sh
$ pnpm dev build
```

_This might take a surprisingly long time_. Thankfully, Docker's build step is _heavily_ cached. It won't take this long again (unless of course you need to change one of those early steps).

## Development

You can run development mode using the following:

```sh
$ pnpm dev
```

This command will run everything in watch mode, and start the Docker containers. However, it will first ask you if you want to build the containers, as in what `pnpm dev build` does. This is asked because building may take a long time.

Once everything has started, you can connect to `http://www.wikijump.localhost/`. Changes you make to the codebase should automatically be applied to the containers, as your machine's filesystem has been "bound" to the containers' filesystem. This is one-way, so a container can't modify your filesystem.

You can just kill the terminal (`CTRL + C` usually) when you want to stop the server.

If for some reason development resources aren't shutting down correctly, you can run the following:

```sh
$ pnpm dev clean
```

If you want to entirely _reset_ the containers, as their data is otherwise persistent even across restarts, you can run the following:

```sh
$ pnpm compose down
```

You can also bring containers up without the JS/TS development mode.
```
$ pnpm dev serve
```

## Using `sudo`

If your environment requires docker commands be run using `sudo`, the normal `pnpm dev` and `pnpm compose` commands will fail. You can use the `sudo` variants of these commands, `pnpm dev sudo` and `pnpm compose-sudo`.

## Entering the container

If you want to enter the container to make temporary changes, you can do so by entering it with a CLI. From Docker Desktop, after running the containers, find the Wikijump app and within it the container you wish to enter, then click the 'CLI' button. Or from the command line:

```
$ docker exec -it [name of container] sh
```

...where `[name of container]` is the name of the container from `docker container ls`.

One reason you may need to enter the container is to manually adjust the Wikijump config. For example, if you use a port other than 80 for your Docker container, you will need to edit `site.custom_domain` to add the port number (e.g. "`www.wikijump.localhost:8080`"). Alternatively, use curl to set the domain directly (e.g. "`-H 'www.wikijump.localhost'`")

## Clock Drift

If you enable multi-factor authentication on a local container you may find that
the clock drift is too great for TOTP codes to work. In docker-compose (`wsl -d docker-compose`)
you can enter this command to sync your time up:

```
$ ntpd -d -q -n -p 0.pool.ntp.org
```

## Deployment via Docker manually

If you wish to build and run your Docker containers manually, consult the commands utilized in `web/nabs.yml` for the various `pnpm` deployment commands. In essence, you will want to build each container for the environment you're targeting, and then launch them. If you're deploying locally (e.g. for development), you can use the `docker-compose.yaml` to run all the containers using Docker Compose, which provides benefits such as volume mapping. This way, directories in your local repository are mapped into the container, allowing any changes to them to immediately appear to the container.

It's useful to keep track of existing Docker images and containers, and destroy them when you no longer need them, so you don't waste space rebuilding the same image over and over. If you are using Docker Desktop, you can manage containers and images from the GUI. Otherwise, using the command line:

```
$ docker container ls  # List containers
$ docker rm [ID]       # Destroy the container with this ID
$ docker images        # List images
$ docker rmi [ID]      # Remove the image with this ID
```

## "Permission denied" error
You might get a PHP "failed to open stream: Permission denied" error when attempting to visit your Wikijump installation. This is because the `www-data` user isn't the owner of the directory Wikijump is trying to write. This shouldn't happen in the first place, but things such as local volume mapping and stuff in the local environment may cause this to happen. If you see this, from Docker Desktop, find the `wikijump_php-fpm_1` or whatever container is hosting PHP-FPM, then run:

```
$ chown -R www-data:www-data directory that is erroring
```

Example:

```
$ chown -R www-data:www-data /var/www/wikijump/web/storage
```

The error should be resolved.
## Relevant Documentation

- [Docker Compose](https://docs.docker.com/compose/)
- [Laravel](https://laravel.com/docs/8.x/)
- [Blade templates](https://laravel.com/docs/8.x/blade)
- [PNPM](https://pnpm.io/)
- [Vite](https://vitejs.dev/)
- [Vitest](https://github.com/vitest-dev/vitest)

# Local Development

This document will explain how to set up Wikijump on your machine for local development.

## Environment

> ### For Windows:
>
> For Windows, you will be using WSL2, a way of running a Linux distribution simultaneously with Windows. You will need Windows 10 for this. The only alternative to WSL2 would be using a Linux virtual machine such as [VirtualBox](https://www.virtualbox.org/) - but using a VM isn't recommended and it won't be explained how to use one here.
>
> It is recommended that you use Ubuntu for the WSL2 distribution. Ubuntu in particular has considerations made for WSL2 use and in general will be the most reliable way forward.
>
> [WSL2 download and installation page](https://docs.microsoft.com/en-us/windows/wsl/install-win10)

## Deployment

The `install` folder has everything you need to run a local Wikijump install either in a container or on metal or a VM.

### Deployment via Docker / pnpm

The recommended way to install Wikijump is via Docker, utilizing pnpm. Docker is a way of containerizing, or in the case of Windows or Mac, also virtualizing, Linux images. It lets you easily create and destroy different Wikijump builds, and it also acts like a sandbox to protect the rest of your system from dependency pollution.

You will need [Docker](https://www.docker.com/) installed and running:

<table>
  <thead><tr><th>Ubuntu / Ubuntu VM</th><th>Windows via WSL2</th></tr></thead>
  <tbody valign="top"><tr>
    <td>
      <p><pre># apt install docker.io</pre></p>
      <p>Start the Docker daemon in another terminal, and leave it running in the background:</p>
      <p><pre>$ dockerd</pre></p>
    </td>
    <td>
      <p>Install <a href="https://docs.docker.com/docker-for-windows/install-windows-home">Docker Desktop</a>, which you'll be using as the Docker daemon, and leave it running in the background.</p>
      <p>If you have set <code>appendWindowsPath=false</code> in your WSL config, then you may hit an error along the lines of <code>"docker-credential-desktop.exe": executable file not found in $PATH</code>. In this case you should either add <code>/mnt/c/Program\ Files/Docker/Docker/resources/bin</code> to your PATH, or <a href="https://github.com/rossjrw/dotfiles/blob/3c5445abb138b735cc3caf61f070c9125fa87d2f/.profile#L28">create a symlink</a> in a directory that is in your PATH already.</p>
    </td>
  </tr></tbody>
</table>

## Setup: Utilites and Programs

You will need some utilites and programs to get started. You will to install the following:

- [PHP 7.4](https://www.php.net/downloads)
- [Node (and NPM, which comes with it), v15 or greater](https://nodejs.org/en/)
- [PNPM v6](https://pnpm.io/installation)
- [Docker](https://www.docker.com/get-started)

Node, NPM, and PNPM are well-behaved on Windows and Linux, and the difference in usage between distributions is negligable.

Then install [Docker Compose](https://docs.docker.com/compose/) and [pnpm](https://pnpm.io/).

## Setup: Configuration

Wikijump has a lot of configuration options that can set. The primary configuration file can be found in `web/conf/`, named `wikijump.ini`. Freshly cloned, your Wikijump repo won't have this file. However, it will have an example configuration, `wikijump.ini.example`, which you can just copy and rename to `wikijump.ini` for the time being.

Second, there is a Docker configuration file that configures the various containers that host Wikijump in the local environment. You can find this file in `install/local/dev/`, named `docker-compose.yaml`. Like before, you won't have this file with a fresh Wikijump clone. However, again like before, there is a `docker-compose.yaml.example`, which you can just copy and rename to `docker-compose.yaml`.

Notice that in `docker-compose.yaml`, there are configuration options for the domains to use. For development purposes, these are set to `wikijump.localhost`. This is the domain you will be connecting to, e.g. `http://www.wikijump.localhost`. The TLD `.localhost` is just like the usual `localhost` domain. Your browser trusts both of these domains implicitly so there is no need to use `HTTPS`.

Finally, you will need a `.env` file in the `web/` root. There is a `.env.example` file that you can copy and rename.

## Setup: Dependencies

You will need to install Wikijump's NPM and Composer dependencies. You should do PHP/Composer first.

Navigate to the root `web` directory and install its dependencies:

```sh
$ pnpm install
```

Now you can utilize `pnpm`'s convenience scripts to deploy locally:

%

## Building

You can now finally build the JS/TS codebase and Docker images using the following command:

```sh
$ pnpm build
```

This will first build the JS/TS codebase, which probably won't take very long at all, and then will build the Docker containers. _This might take a surprisingly long time_. Thankfully, Docker's build step is _heavily_ cached. It won't take this long again (unless of course you need to change one of those early steps).

## Development

You can run development mode using the following:

```sh
$ pnpm dev
```

This command will run everything in watch-mode, and start the Docker containers. However, it will first ask you if you want to build the containers, as in what `pnpm build` does. This is asked because building may take a long time.

Once everything has started, you can connect to `http://www.wikijump.localhost/`. Changes you make to the codebase should automatically be applied to the containers, as your machine's filesystem has been "bound" to the containers' filesystem. This is one-way, so a container can't modify your filesystem.

You can just kill the terminal (`CTRL + C` usually) when you want to stop the server. (If you're on Windows, you may want to _hold_ `CTRL + C` to bypass annoying `Terminate batch job?` prompts, and if you do so, make sure to wait for the Docker containers to exit before restarting the development mode.)

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

### Deployment via Docker manually

If you wish to build and run your Docker containers manually, consult the commands utilized in `web/nabs.yaml` for the various `pnpm` deployment commands. In essence, you will want to build each container for the environment you're targeting, and then launch them. If you're deploying locally (e.g. for development), you can use the `docker-compose.yaml` to run all the containers using Docker Compose, which provides benefits such as volume mapping. This way, directories in your local repository are mapped into the container, allowing any changes to them to immediately appear to the container.

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

If you want to enter the container to make temporary changes, you can do so by entering it with a CLI. From Docker Desktop, after running the containers, find the Wikijump app and within it the container you wish to enter, then click the 'CLI' button. Or from the command line:

```
$ docker exec -it [name of container] sh
```

...where `[name of container]` is the name of the container from `docker container ls`.

One reason you may need to enter the container is to adjust the Wikijump config. For example, if you use a port other than 80 for your Docker container, you will need to edit `site.custom_domain` to add the port number (e.g. "`www.wikijump.localhost:8080`"). Alternatively, use curl to set the domain directly (e.g. "`-H 'www.wikijump.localhost'`")

## Relevant Documentation

- [Laravel](https://laravel.com/docs/8.x/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Blade templates](https://laravel.com/docs/8.x/blade)
- [PNPM](https://pnpm.io/)
- [Vite](https://vitejs.dev/)

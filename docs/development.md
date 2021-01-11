# Local Development

This document will explain how to set up Wikijump on your machine for local development.

## Environment

### Operating system

To run Wikijump, you will need to be running Linux.

If you are running Windows, you have two options:

* Install a Virtual Machine (VM) running Linux, for example using [VirtualBox](https://www.virtualbox.org/), and work from inside it as if it were Linux
* (Recommended) Install [Windows Subsystem for Linux 2 (WSL2)](https://docs.microsoft.com/en-us/windows/wsl/install-win10)

In either case, when choosing a Linux distribution, Ubuntu is fine.

### Connecting to localhost

You will need edit your system's HOSTS file so that when you navigate to https://www.wikijump.test, your browser knows that you want to connect to your own virtual server and not somewhere on the internet. On Linux, your HOSTS file is `/etc/hosts`. On Windows, it is `C:\Windows\System32\drivers\etc\hosts`, but in order to edit it you will need to be running an editor (e.g. Notepad) as administrator. If you are running WSL2, edit your Windows HOSTS file; ignore your Linux HOSTS file.

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

You will also need to add a line for any wiki that you create, e.g. `127.0.0.1 my-new-wiki.wikijump.test`.

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

Once those are configured, you can use the provided `docker-compose.yaml` file to get the containers started. The following will build all the images, and then run new containers with the prefix `wikijump`:

```
$ cd install
$ docker-compose -p wikijump up
```

When running, navigate to https://www.wikijump.test/ in your browser. This will contain the containerized Wikijump installation.
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

## Configuration

If you installed Wikijump directly to your machine with `legacy/install.sh`, you can edit the Wikijump config at any point. If you installed Wikijump via Docker, you will need to enter the container to edit the config:

```
$ docker exec -it wj bash
```

You will likely want to set `allow_http` in `/var/www/conf/wikijump.ini` to `true`, since HTTPS locally is troublesome.

If you use a port other than 80 for your Docker container, you will need to edit `site.custom_domain` to add the port number (e.g. "`www.wikijump.test:8080`"). Alternatively, use curl to set the domain directly (e.g. "`-H 'www.wikijump.test'`")

After editing the Wikijump config, you may need to restart nginx.

On Linux with systemd:

```
# systemctl restart nginx
```

On Linux with sysv-init, Windows via WSL2, or from within a Docker container:

```
# service nginx restart
```

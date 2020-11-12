# Local Development

The `installer` folder has everything you need to run a local Wikijump install either in a container or on metal or a VM.

### Installation

You can create a docker container using the following:

```bash
$ docker build . -t scpwiki/wikijump:local
$ docker create --name wikijump -p 8080:80 scpwiki/wikijump:local
$ docker start wikijump
```

Instead of building Wikijump locally, you can also pull the image from the Docker Hub:

```bash
$ docker create --name wikijump -p 8080:80 scpwiki/wikijump:latest
```

Then terminate it:

```bash
$ docker stop wikijump
```

Alternatively, you can install to your local system using `install.sh`. This may require tinkering depending on your exact platform and environment.

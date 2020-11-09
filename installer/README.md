# wikijump-infra
Scripts, tools, and snippets to power [Wikijump](https://github.com/scpwiki/wikijump), our fork of the unmaintained [Wikidot](https://github.com/gabrys/wikidot).

This contains the necessary resources to create an environment capable of bootstrapping a Wikijump server. It provides a seed migration for the database, configuration files, and list of dependencies.

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

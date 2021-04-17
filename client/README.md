## Wikijump Client Monorepo

### Relevant Documentation

* [Lerna](https://github.com/lerna/lerna)
* [`uvu`](https://github.com/lukeed/uvu)
* [Snowpack](https://www.snowpack.dev/)

### Getting Started

You should be running a recent version of Node and NPM. Specifically, this repo has been made with Node v15 and NPM v7. It'll likely work on some earlier versions, regardless.

To actually setup the repo, run this command:

```
$ npm run update
```

This will install dependencies, clean package modules, bootstrap, and then build all packages.

Run `npm run bootstrap` any time you need to ensure that packages have their dependencies sorted out.

### Building

```
$ npm run build
```

### Development

```
$ npm run dev
```

Development mode will begin with a full build in order to ensure that dependency resolution succeeds. After that, a development process will be started and packages will be rebuilt every time a file of theirs changes.

### Commands

See [`useful-commands.md`](docs/useful-commands.md).

### Modules, Websites

See [`modules.md`](docs/modules.md), and [`websites.md`](docs/websites.md).

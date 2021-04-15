## Wikijump Client Monorepo

The design of this monorepo is incomplete. Agreement on a testing framework and architecture, and a few other minor details are needed to advance the design.

### Relevant Documentation

* [Lerna](https://github.com/lerna/lerna)

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

See [`useful-commands.md`.](docs/useful-commands.md)

### Creating a Package

Creating a package is very simple. Navigate to the `templates/module-template` folder. There, you will find a premade template package that can be dropped into `modules/`. Make sure to edit the `package.json` to fit the package you intend to create, e.g. `name` and `description` fields.

One fairly manual tweak you need to make is adding the package directory to the `references` field in the root `tsconfig.json`. This is to make sure that `.d.ts` declarations are compiled - which are essential to typecheck the monorepo.

After that, everything else is automatic. Your package will be automatically processed by Lerna.

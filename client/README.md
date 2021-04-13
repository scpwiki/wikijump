## Wikijump Client Monorepo

The design of this monorepo is incomplete. Agreement on linting rules, testing framework and architecture, and a few other minor details are needed to advance the design.

### Relevant Documentation

* [Lerna](https://github.com/lerna/lerna)

### Getting Started

TODO: NPM, Node requirements

```
$ npm i
$ npm run bootstrap
```

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
```
$ npm run boostrap    Link packages together, hoist shared dependencies.
$ npm run clean       Cleanup packages node_modules and TSC build data.
$ npm run build       Build all packages.
$ npm run dev         Start a devlopment server for rebuilding packages when they change.
```

### Creating a Package

Creating a package is very simple. Navigate to the `docs/package-template` folder. There, you will find a premade template package that can be dropped into `packages/`. Make sure to edit the `package.json` to fit the package you intend to create, e.g. `name` and `description` fields.

One fairly manual tweak you need to make is adding the package directory to the `references` field in the root `tsconfig.json`. This is to make sure that `.d.ts` declarations are compiled - which are essential to typecheck the monorepo.

After that, everything else is automatic. Your package will be automatically processed by Lerna.

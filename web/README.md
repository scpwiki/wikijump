## Wikijump Web

> #### **IMPORTANT:**
>
> This monorepo uses [PNPM](https://pnpm.io/). You need to have it installed.
>
> ```
> $ npm install pnpm --global
> ```
>
> You will need to use PNPM to interact with this monorepo, and not `npm`.

### Relevant Documentation

- [PNPM](https://pnpm.io/)
- [Mocha](https://mochajs.org)
- [Vite](https://vitejs.dev/)

### Getting Started

You will need a recent version of Node, specifically v15 or higher. The repo may work on older versions of Node but this hasn't been tested. You will also need PNPM, as noted above.

To get setup, run this command:

```
$ pnpm install
```

This will have PNPM install all the needed dependencies, even for workspace packages.

### Building

To build the monorepo, run this command:

```
$ pnpm build
```

This will build all packages for which a build process has been setup. Most packages within the repo don't actually have a build step - this is because packages are usually intended to be consumed by something like a website bundler, and it would be mostly pointless to build the package if it would be built by the website bundler anyways.

> Packages in the repo tend to be designed for direct consumption by Vite - and thus use Vite's extra language features, such as URL imports. This makes them directly incompatible with being used in another environment, without some sort of Vite-based build step.

### Packaging

To "package" a module, run this command:

```
$ pnpm modules:pack -- module-name
```

This will "package" the module `module-name`. What this does is that it starts a special build process on that module, which will yield a publishable NPM package in the modules `dist` folder. Any important changes that would've needed to been made to the module and its `package.json` are handled automatically. You won't normally need to call this command, as the package publishing step does this for you. However, it's useful to make sure that the package can actually build.

### Versioning

There are two commands for versioning:

```
$ pnpm changeset
$ pnpm modules:version
```

`changset` starts the Changesets wizard for adding new changelogs for a package. Follow what the wizard says. Changesets will handle version bumping and changelogs for you.

`modules:version` bumps all the versions via the changelogs Changesets has updated.

### Publishing

To publish modules, run this command:

```
$ pnpm modules:publish
```

This will start the publishing process for any modules whose version is higher than what is currently in the NPM registry. Make sure you are absolutely positive you want to take this step, and make sure you've ran `pnpm modules:version`.

### Development

See the [`frontend-development`](../docs/frontend-development.md) docs.

### Testing

Running tests is simple:

```
$ pnpm test
# pnpm cover
```

All tests are ran in a browser, compiled by Vite. Liberal use of Vite's file import features, such as import globs, is recommended when making tests.

At the end of a code coverage test, a report will be emitted. This coverage report should only contain source TypeScript and Svelte files - if it reports coverage for something that isn't one of those two, something has gone wrong. If it reports nothing, that also means something has gone wrong.

> #### **IMPORTANT:**
>
> If you need to use c8 code coverage comments, e.g:
>
> ```js
> /* c8 ignore next */
> ```
>
> You need to do the following instead:
>
> ```js
> /*! c8 ignore next */
> // note the exclamation point
> ```
>
> This is for technical reasons - build tools don't preserve normal comments but will preserve legal comments. These comments are automatically transformed by the test builder back into normal comments - but you need to mark them as "important" using that exclamation point.

### Linting, Validation

To lint the entire monorepo, simply do:

```
$ pnpm lint

or, if you want auto-fixes applied:
$ pnpm lint:fix
```

This will run ESLint, Prettier, and Stylelint. You can run a linter individually by appending `lint` (or `lint:fix`) with the name of the linter, e.g. `lint:eslint`.

You can run `tsc` (TypeScript) for typechecking with:

```
pnpm typecheck
```

If you want to run linting and typechecking at the same time, and have everything done in parallel, do:

```
pnpm validate
```

### Commands

Commands are just NPM scripts, defined in `nabs.yml` and then compiled to
`package.json` with `pnpm nabs`.

### Modules

See [`modules.md`](../docs/modules.md).

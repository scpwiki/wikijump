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

- [Docker Compose](https://docs.docker.com/compose/)
- [Laravel](https://laravel.com/docs/8.x/)
- [Blade templates](https://laravel.com/docs/8.x/blade)
- [PNPM](https://pnpm.io/)
- [Vite](https://vitejs.dev/)
- [Vitest](https://github.com/vitest-dev/vitest)

### Getting Started

See [`development.md`](../docs/development.md)

### Development

To run the development CLI, do:
```
$ pnpm dev
```

There are extra arguments you can provide:
```
sudo  : runs `docker-compose` commands with `sudo`
build : builds the containers, but doesn't launch anything
serve : launches the server, but doesn't launch Vite
clean : cleans up any running services
```

### Modules

Modules can be found in the [`modules`](./modules) folder. See [`modules.md`](../docs/modules.md).


### Testing

Running tests is simple:

```
$ pnpm test        # run tests
$ pnpm test:watch  # rerun tests whenever a file changes
$ pnpm test:ui     # open Vitest UI
$ pnpm cover       # run tests, get coverage report
```

Tests are ran in Node via Vitest. Liberal use of Vite's file import features, such as import globs, is recommended when making tests.

At the end of a code coverage test, a report will be emitted. This coverage report should only contain source TypeScript and Svelte files - if it reports coverage for something that isn't one of those two, something has gone wrong. If it reports nothing, that also means something has gone wrong.

> #### **IMPORTANT:**
>
> If you need to use c8 code coverage comments, e.g:
>
> ```js
> /* c8 ignore next */
> ```
>
> you simply can't. This is due to the current limitations of Vitest.

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
$ pnpm typecheck
```

If you want to run linting and typechecking at the same time, and have everything done in parallel, do:

```
$ pnpm validate
```

### Dependencies

You can run a command to check if dependencies need updating:
```
$ pnpm taze
```

The command is named `taze` because that's the dependency checking utility being used.

Once you've reviewed what dependencies need updating (make sure updating them won't break anything), you can write to each `package.json` using:
```
$ pnpm taze:write

don't forget to update the lock file:
$ pnpm install
```

This will update all dependencies to their latest version.

The file `.tazerc.json` will lets you exclude dependencies from being checked and updated, so you can modify that if needed. This is especially useful if you've pinned a dependency.

### Commands

Commands are just NPM scripts, defined in `nabs.yml` and then compiled to
`package.json` with `pnpm nabs`.

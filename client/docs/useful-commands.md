```
# Basic
$ npm run update      Install dependencies, clean packages, boostrap, then build.
$ npm i               Install the root's packages. Make sure to run 'npm run bootstrap' afterwards.
$ npm run boostrap    Link packages together, hoist shared dependencies.
$ npm run clean       Cleanup packages node_modules and tsc build data.
$ npm run build       Build all packages.
$ npm run dev         Start a devlopment server for rebuilding packages when they change.

# Lerna (not exhaustive)
$ npx lerna add       See Lerna docs. Adds a dependency to package(s) in a monorepo context.
$ npx lerna run       See Lerna docs. Runs a script on package(s)

# Advanced
$ npm run build:dts   Build package TypeScript declarations.
$ npm run build:es    Build package bundles.
```

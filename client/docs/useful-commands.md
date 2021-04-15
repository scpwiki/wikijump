```
$ npm run update         Install dependencies, clean packages, boostrap, then build.
$ npm i                  Install the root's packages. Be sure to run 'npm run bootstrap'.
$ npm run boostrap       Link packages together, hoist shared dependencies.
$ npm run clean          Cleanup packages node_modules and tsc build data.
$ npm run build          Build all packages.
$ npm run dev            Start a dev. server for rebuilding packages when they change.
$ npm run test           Build all tests, and then run them.

# Lerna (not exhaustive)
$ npx lerna add          See Lerna docs. Adds a dependency to package(s).
$ npx lerna run          See Lerna docs. Runs a script on package(s)

# Linting
$ npm run eslint         Checks packages for lint errors. Doesn't autofix.
$ npm run eslint-fix     Checks packages for lint errors and fixes them automatically.
$ npm run prettier       Checks packages for style errors. Doesn't autofix.
$ npm run prettier-fix   Checks packages for style errors and fixes them automatically.
```

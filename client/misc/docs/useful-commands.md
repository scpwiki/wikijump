# Useful commands

This page lists some commands that will be necessary for development.

Most commands are NPM scripts, defined in `nabs.yml` and then compiled to 
`package.json`.

## Installing dependencies

- `npm i` - Install the root's dependencies. Be sure to run 'npm run 
  bootstrap' afterwards.
- `npm run update` - Installs all dependencies throughout the monorepo, 
  bootstraps package dependencies, and then builds all packages.

## Monorepo management

The `client/` monorepo is managed with [Lerna](https://lerna.js.org/).

- `npm run bootstrap` - Link packages together, hoist shared dependencies.
- `npx lerna add` - Adds a dependency to package(s).

## Development server

Opening a development server for a website will start a process that 
exposes the website to a local URL which you can access in your browser. 
Editing any file that the website depends on will automatically reload the 
page without you needing to do anything.

There is a separate development server command for each website.

- `npm run dev:dev-sandbox` - Open the dev server for the dev-sandbox.

## Building

- `npm run build` - Build all packages.

## Testing

- `npm run test` - Build and run all tests.

## Linting

[ESLint](https://eslint.org/) is used for code linting, and
[Prettier](https://prettier.io/) for style formatting.

- `npm run lint` - Check for lint and style issues.
- `npm run lint:fix` - Check for and resolve lint and style issues.

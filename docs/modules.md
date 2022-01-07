# Monorepo: Modules

Modules are the packages that can be found in the `modules/` folder. They're intended as self-contained ESM available to other packages, such as a website or build tool. They're usually uncompiled/unbuilt, being consumed directly from their source.

They have an important file structure (don't change it), which looks like the following:

```
module:
- src/
  - index.ts
  - **/*.ts
- tests/
  - *.ts
- package.json
- README.md
- tsconfig.json
```

Other than the inheritance of development dependencies and tooling configurations, these modules act like fairly normal NPM packages. If you need to add a dependency to one, you can either navigate to the module and do a normal `pnpm add <dep>` command, or manually add it to the `package.json`.

There is a template for making modules, and it can be found in the `module-template` folder.

Here is how to use that template:

1. Create a folder in `modules/` that is the name of your package.
   It isn't strictly required that it is the name of your package, but it gets confusing otherwise.

2. Copy the contents of `module-template` into the new folder.

3. Edit `package.json`.
   There are a few properties you should edit. They are: `name`, `description`, and `version`. Everything else is fine being left alone. Make sure `name` is prefixed with `@wikijump/`.

4. Edit `README.md`.
   This one is fairly self-explanatory: edit the readme to display the correct module name and add any important details that you feel should be added.

5. Go into `tests/` and delete, or reuse, the `test-template.ts` file.
   This is a file demonstrating the basics of how to use the test framework. Tests are built and ran automatically.

6. You're done!

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

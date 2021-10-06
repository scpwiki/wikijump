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

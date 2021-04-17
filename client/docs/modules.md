# Monorepo: Modules

Modules are the packages that can be found in the `modules/` folder. They're intended for pure ESM output, and are compiled rapidly using Estrella, an esbuild-based utility. They have an important file structure (don't change it), which looks like the following:

```
module:
- src/
  - index.ts
  - **/*.ts
- tests/
  - *.ts
- package.json
- tsconfig.json
```

The `src/` folder is compiled into a `dist/` folder when the module is built. Similarly, files in the `tests/` folder are compiled into the `tests/dist/` folder when the tests are built.

Other than the inheritance of development dependencies and tooling configurations, these modules act like fairly normal NPM packages. If you need to add a dependency to one, you can either navigate to the module and do a normal `npm i <dep>` command, or manually add it to the `package.json`. After you do that, you need to make sure to run the `npm run bootstrap` command at the root. This ensures that all the fragile symlinks get restored by Lerna.

To improve performance of the monorepo, building and watching of modules isn't done per module, but instead globally at root. It is recommended that you just build the entire monorepo rather than trying to build individual packages. This isn't a performance problem because the repo builds modules incrementally. If a module has not changed, it won't get rebuilt.

There is a template for making modules, and it can be found in the `templates/module-template` folder.

Here is how to use that template:

1. Create a folder in `modules/` that is the name of your package.
  It isn't strictly required that it is the name of your package, but it gets confusing otherwise.
  If you want to build the module as a Node package instead of for the browser, prefix the folder name with `node-`.

2. Copy the contents of `templates/module-template` into the new folder.

3. Edit `package.json`.
  There are a few properties you should edit. They are: `name`, `description`, and `version`. Everything else is fine being left alone.

4. Go into `tests/` and delete, or reuse, the `test-template.ts` file. This is a file demonstrating the basics of how to use the simple [`uvu` testing framework](https://github.com/lukeed/uvu).

5. Go to the root `tsconfig.json` and edit the `references` property, and add your module to it. This has to be done manually, unfortunately. If you don't do this, your module won't have any `.d.ts` files generated.

6. You're done!

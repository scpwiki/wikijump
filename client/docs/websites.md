# Monorepo: Websites

This monorepo has special support for website packages, in the `web/` folder. These are quite a bit more complex than module packages. While technically there is no specific restriction on what can be used to build or develop a website package, it is recommended that you use [Snowpack](https://www.snowpack.dev/). This documentation will assume that you're using Snowpack and that you started with the `templates/website-template` configuration.

The file structure will look like this:
```
website:
- public/ (might have to be created)
  - **/*
- src/
  - index.html
  - **/*
- package.json
- snowpack.config.cjs
- tsconfig.json
```

The `public` folder is where source files that do not need compiling or transformations by Snowpack are kept. The `src` folder is where the rest of the source files go, such as TypeScript, CSS, etc. The file structure of these two folders are compiled/built into a `dist/` folder by Snowpack.

Other than the inheritance of development dependencies and tooling configurations, websites act like fairly normal NPM packages. If you need to add a dependency to one, you can either navigate to the website package and do a normal `npm i <dep>` command, or manually add it to the `package.json`. After you do that, you need to make sure to run the `npm run bootstrap` command at the root. This ensures that all the fragile symlinks get restored by Lerna.


There is a template for making websites, and it can be found in the `templates/website-template` folder.

1. Create a folder in `web/` named the same as the website's package name.

2. Copy the contents of `templates/website-template` into the new folder.

3. Edit `package.json`, and change the `name` field to the name of this package.

4. You're done!

You may want to add a command in the root `package.json` to make it easier to start the build and watch modes for your website package. Look at the `dev-sandbox` scripts, and duplicate how those work.

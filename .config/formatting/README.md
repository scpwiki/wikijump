# @wikijump/formatting

This module neatly organizes Wikijump's formatting infrastructure into a single module.

### `package.json`

```json
{
  "prettier": "@wikijump/formatting",
  "eslintConfig": { "extends": "./.eslintrc.yaml" },
  "stylelint": { "extends": "./stylelintrc.json" }
}
```

You'll need to change the relative paths to match the location of your `package.json`. Additionally, you'll need to add `@wikijump/formatting` as a dev dependency.

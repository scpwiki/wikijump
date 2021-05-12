module.exports = {
  root: true,

  extends: "../client/.eslintrc.js",

  ignorePatterns: ["**/node_modules/**", "**/vendor/**", "**/dist/**"],

  parserOptions: {
    sourceType: "module",
    tsconfigRootDir: __dirname,
    project: ["./tsconfig.json"],
    extraFileExtensions: [".svelte"]
  }
}

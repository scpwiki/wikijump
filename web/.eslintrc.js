module.exports = {
  env: {
    es2020: true,
    browser: true
  },
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/eslint-recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking"
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    parser: "@typescript-eslint/parser",
    project: "./tsconfig.json",
    tsconfigRootDir: __dirname,
    sourceType: "module",
    lib: ["es2020", "dom", "dom.iterable"]
  },
  plugins: [
    "@typescript-eslint/eslint-plugin"
  ],
  rules: {
    quotes: "off",
    'prefer-template': "error",
    '@typescript-eslint/no-unused-vars': [
      "error", { argsIgnorePattern: "^_" }
    ],
    'space-before-blocks': "error",
    'space-before-function-paren': "error",
    // Enforce semicolons
    semi: "off",
    '@typescript-eslint/semi': ["error", "always"],
    '@typescript-eslint/member-delimiter-style': [
      "error", {
        multiline: { delimiter: "semi" },
        singleline: { delimiter: "semi" }
      }
    ],
    // Temporary rules to be removed when Wikijump is type-safe
    '@typescript-eslint/no-non-null-assertion': "off",
    '@typescript-eslint/explicit-module-boundary-types': "error",
    '@typescript-eslint/no-unsafe-assignment': "warn",
    '@typescript-eslint/no-unsafe-member-access': "warn",
    '@typescript-eslint/no-unsafe-call': "warn",
    '@typescript-eslint/restrict-template-expressions': "warn",
    '@typescript-eslint/no-explicit-any': "warn",
  }
}

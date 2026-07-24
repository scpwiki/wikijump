import eslint from "@eslint/js";
import globals from "globals";

export default [
  {
    ignores: ["artifacts/**"],
  },
  eslint.configs.recommended,
  {
    files: ["**/*.{js,mjs,cjs}"],
    languageOptions: {
      ecmaVersion: "latest",
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      sourceType: "module",
    },
    rules: {
      // Control-character regexes are intentional input validators in this tooling.
      "no-control-regex": "off",
    },
  },
];

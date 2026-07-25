import js from "@eslint/js"
import svelte from "eslint-plugin-svelte"
import globals from "globals"
import parser from "svelte-eslint-parser"
import ts from "typescript-eslint"

import { defineConfig, globalIgnores } from "eslint/config"

export default defineConfig(
  js.configs.recommended,
  {
    rules: {
      // these rules are deprecated and replaced by @stylistic/eslint-plugin
      // "template-curly-spacing": "error",
      // "wrap-iife": "error",
      // "new-parens": "warn",
      eqeqeq: "error",
      yoda: "error",
      "prefer-rest-params": "error",
      "prefer-spread": "error",
      "symbol-description": "error",
      "prefer-numeric-literals": "error",
      "prefer-template": "error",
      "no-useless-rename": "error",
      "no-useless-computed-key": "error",
      "no-useless-concat": "error",
      "no-undef-init": "error",
      "no-throw-literal": "error",
      "default-case-last": "error",
      "prefer-arrow-callback": ["error", { allowNamedFunctions: true }],
      "no-alert": "error",
      "no-caller": "error",
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-var": "error",
      "no-script-url": "error",
      "no-lonely-if": "warn",
      "no-unneeded-ternary": "warn",
      "operator-assignment": "warn",
      "prefer-exponentiation-operator": "warn",
      curly: ["warn", "multi-line"],

      // this rule interferes with setting bindable component prop in svelte
      "no-useless-assignment": "warn"
    }
  },
  ts.configs.recommended,
  {
    rules: {
      // these rules are deprecated and replaced by @stylistic/eslint-plugin.
      // "@typescript-eslint/space-infix-ops": ["warn", { int32Hint: true }],

      // this rule is deprecated, see https://typescript-eslint.io/rules/ban-types/
      // "@typescript-eslint/ban-types": "error",

      // these rules are enabled in recommended config.
      // "@typescript-eslint/no-misused-new": "error",
      // "@typescript-eslint/no-non-null-asserted-optional-chain": "error",
      // "@typescript-eslint/no-require-imports": "error",
      // "@typescript-eslint/no-this-alias": "error",
      // "@typescript-eslint/no-extra-non-null-assertion": "error",
      // "@typescript-eslint/no-unnecessary-type-constraint": "error",
      // "@typescript-eslint/prefer-as-const": "error",
      // "@typescript-eslint/prefer-namespace-keyword": "error",

      "@typescript-eslint/no-for-in-array": "error",
      "@typescript-eslint/prefer-optional-chain": "error",
      "@typescript-eslint/prefer-regexp-exec": "error",
      "no-useless-constructor": "off",
      "@typescript-eslint/no-useless-constructor": "error",
      "@typescript-eslint/unbound-method": "error",
      "@typescript-eslint/triple-slash-reference": ["error", { types: "prefer-import" }],
      "@typescript-eslint/adjacent-overload-signatures": "warn",
      "@typescript-eslint/array-type": "warn",
      "@typescript-eslint/no-inferrable-types": "warn",
      "@typescript-eslint/consistent-indexed-object-style": "warn",
      "@typescript-eslint/no-confusing-non-null-assertion": "warn",
      "@typescript-eslint/class-literal-property-style": ["warn", "fields"],
      "@typescript-eslint/consistent-type-exports": [
        "warn",
        { fixMixedExportsWithInlineTypeSpecifier: true }
      ],

      // TODO: warn no-explicit-any and no-unused-vars
      // should remove them later
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": "warn"
    }
  },
  ...svelte.configs.recommended,
  {
    files: ["**/*.svelte"],
    rules: {
      // these rules are enabled in recommended config.
      // "svelte/no-dupe-else-if-blocks": "error",
      // "svelte/no-dupe-style-properties": "error",
      // "svelte/no-not-function-handler": "error",
      // "svelte/no-object-in-text-mustaches": "error",
      // "svelte/no-shorthand-style-property-overrides": "error",
      // "svelte/no-store-async": "error",
      // "svelte/valid-prop-names-in-kit-pages": "error",

      // this rule is deprecated, see https://sveltejs.github.io/eslint-plugin-svelte/rules/no-dynamic-slot-name/
      // "svelte/no-dynamic-slot-name": "error",

      "svelte/valid-compile": "error",
      "svelte/no-target-blank": "error",

      "svelte/require-store-callbacks-use-set-param": "warn",
      "svelte/button-has-type": "warn",
      "svelte/no-at-debug-tags": "warn",
      "svelte/no-reactive-functions": "warn",
      "svelte/no-reactive-literals": "warn",
      "svelte/no-unused-svelte-ignore": "warn",
      "svelte/no-useless-mustaches": "warn",
      "svelte/derived-has-same-inputs-outputs": "warn",
      "svelte/html-self-closing": "warn",
      "svelte/no-extra-reactive-curlies": "warn",
      "svelte/prefer-class-directive": "warn",
      "svelte/prefer-style-directive": "warn",
      "svelte/shorthand-attribute": "warn",
      "svelte/shorthand-directive": "warn",
      "svelte/sort-attributes": "warn",
      "svelte/spaced-html-comment": "warn",

      // Bindable props are observable by their parent even when not read again locally.
      "no-useless-assignment": "off",
      "svelte/no-at-html-tags": "off"
    }
  },
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2026
      },

      parser: ts.parser,
      ecmaVersion: "latest",
      sourceType: "module",

      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"]
      }
    }
  },
  [
    globalIgnores([
      "**/node_modules/**/*",
      ".desloppify/**/*",
      "./build/**/*",
      "./svelte-kit/**/*",
      "./package/**/*",
      "playwright-report/**/*",
      "test-results/**/*",
      "**/.DS_Store",
      "**/node_modules",
      "build",
      ".svelte-kit",
      "package",
      "**/.env",
      "**/.env.*",
      "!**/.env.example",
      "**/pnpm-lock.yaml",
      "**/package-lock.json",
      "**/yarn.lock",
      "svelte.config.js"
    ]),

    {
      files: ["**/*.js", "**/*.cjs"],

      languageOptions: {
        ecmaVersion: "latest",
        sourceType: "script",

        parserOptions: {
          createDefaultProgram: true
        }
      },

      rules: {
        "@typescript-eslint/no-require-imports": "off"
      }
    },
    {
      files: ["**/*.svelte"],

      languageOptions: {
        parser,
        ecmaVersion: 5,
        sourceType: "script",

        parserOptions: {
          parser: ts.parser
        }
      }
    }
  ]
)

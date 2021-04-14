function useDefault(type, rules) {
  return Object.assign({}, ...rules.map(rule => ({ [rule]: type })))
}

function prefixKeys(prefix, obj) {
  const mappedObj = {}
  for (const key in obj) {
    mappedObj[prefix + key] = obj[key]
  }
  return mappedObj
}

const rules = {
  code: {
    ...useDefault("error", [
      "eqeqeq",
      "yoda",
      "prefer-rest-params",
      "prefer-spread",
      "symbol-description",
      "template-curly-spacing",
      "prefer-numeric-literals",
      "prefer-template",
      "no-useless-rename",
      "no-useless-computed-key",
      "no-useless-concat",
      "no-undef-init",
      "no-throw-literal",
      "default-case-last",
      "wrap-iife"
    ]),
    "prefer-arrow-callback": ["error", { allowNamedFunctions: true }]
  },

  restrict: {
    ...useDefault("error", [
      "no-alert",
      "no-caller",
      "no-eval",
      "no-implied-eval",
      "no-var",
      "no-script-url"
    ])
  },

  style: {
    "prettier/prettier": ["warn"],
    ...useDefault("warn", [
      "new-parens",
      "no-lonely-if",
      "no-unneeded-ternary",
      "operator-assignment",
      "prefer-exponentiation-operator"
    ]),
    "curly": ["warn", "multi-line"],
    "@typescript-eslint/space-infix-ops": ["warn", { int32Hint: true }]
  },

  typescript: {
    ...prefixKeys("@typescript-eslint/", {
      // code
      ...useDefault("error", [
        "ban-types",
        "no-invalid-void-type",
        "no-misused-new",
        "no-non-null-asserted-optional-chain",
        "no-require-imports",
        "no-this-alias",
        "no-extra-non-null-assertion",
        "no-unnecessary-type-constraint",
        "no-for-in-array",
        "prefer-as-const",
        "prefer-namespace-keyword",
        "prefer-optional-chain",
        "prefer-regexp-exec",
        "no-useless-constructor",
        "unbound-method"
      ]),
      "triple-slash-reference": ["error", { types: "prefer-import" }],
      // style
      ...useDefault("warn", [
        "adjacent-overload-signatures",
        "array-type",
        "no-inferrable-types",
        "consistent-indexed-object-style",
        "no-confusing-non-null-assertion"
      ]),
      "class-literal-property-style": ["warn", "fields"]
    })
  },

  typechecked: {
    // code
    ...useDefault("error", [
      "no-misused-promises",
      "no-floating-promises",
      "require-await",
      "no-unnecessary-boolean-literal-compare",
      "no-unnecessary-condition",
      "no-unnecessary-type-assertion",
      "no-confusing-void-expression",
      "no-unnecessary-qualifier",
      "no-unnecessary-type-arguments",
      "non-nullable-type-assertion-style",
      "prefer-includes",
      "prefer-nullish-coalescing"
    ]),
    // style
    ...useDefault("warn", ["dot-notation"])
  },

  regex: {
    ...prefixKeys("clean-regex/", {
      // code
      ...useDefault("error", [
        "confusing-quantifier",
        "no-empty-alternative",
        "no-empty-backreference",
        "no-obscure-range",
        "no-octal-escape",
        "no-optional-assertion",
        "no-unnecessary-assertions",
        "no-zero-quantifier",
        "optimal-lookaround-quantifier"
      ]),

      // style
      ...useDefault("warn", [
        "identity-escape",
        "no-trivially-nested-lookaround",
        "no-trivially-nested-quantifier",
        "no-unnecessary-character-class",
        "no-unnecessary-lazy",
        "no-unnecessary-quantifier",
        "optimal-concatenation-quantifier",
        "optimized-character-class",
        "prefer-character-class",
        "prefer-predefined-character-set",
        "prefer-predefined-quantifiers",
        "simple-constant-quantifier",
        "sort-flags"
      ]),

      "consistent-match-all-characters": ["warn", { charClass: "[^]" }]
    })
  }
}

const baseRules = { ...rules.code, ...rules.restrict, ...rules.style, ...rules.regex }
const typeRules = { ...rules.typescript, ...rules.typeChecked }

module.exports = {
  root: true,
  ignorePatterns: ["**/node_modules/**", "**/dist/**"],
  plugins: ["prettier", "svelte3", "@typescript-eslint", "clean-regex"],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    sourceType: "module",
    tsconfigRootDir: __dirname,
    project: [
      "./tsconfig.json",
      "./modules/*/tsconfig.json",
      "./templates/module-template/tsconfig.json"
    ],
    extraFileExtensions: [".svelte"]
  },
  overrides: [
    // JavaScript (Node)
    {
      files: ["*.js", "*.cjs"],
      env: { node: true, es2021: true },
      parserOptions: { createDefaultProgram: true },
      rules: baseRules
    },
    // JavaScript (Browser)
    {
      files: ["*.mjs"],
      env: { browser: true, es2021: true },
      parserOptions: { createDefaultProgram: true },
      rules: baseRules
    },
    // TypeScript (Browser)
    {
      files: ["*.d.ts", "*.ts", "*.tsx"],
      env: { browser: true, es2021: true },
      rules: { ...baseRules, ...typeRules }
    },
    // Svelte + TypeScript (Browser)
    {
      files: ["*.svelte"],
      processor: "svelte3/svelte3",
      env: { browser: true, es2021: true },
      rules: { ...baseRules, ...typeRules },
      settings: {
        "svelte3/typescript": require("typescript"),
        "svelte3/ignore-styles": () => true
      }
    }
  ]
}

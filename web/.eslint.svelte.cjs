// Svelte + TypeScript (Browser)
// File is separate because Svelte ESLint has a stupid settings
// system that requires `ignore-styles` to be a straight-up JS function
module.exports = {
  overrides: [
    {
      files: ["*.svelte"],
      processor: "svelte3/svelte3",
      env: { browser: true, es2021: true },
      settings: {
        "svelte3/typescript": true,
        "svelte3/ignore-styles": () => true
      }
    }
  ]
}

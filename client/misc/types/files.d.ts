/**
 *  @file Modules for correctly typing imported files.
 */

// -- SPECIAL

declare module "*.css"

declare module "*.svelte"

declare module "*.worker.ts" {
  const text: string
  export default text
}

// -- URL REFERENCES

declare module "*?url" {
  const url: string
  export default url
}

declare module "*.wasm" {
  const url: string
  export default url
}

declare module "*.svg" {
  const url: string
  export default url
}

declare module "*.bmp" {
  const url: string
  export default url
}

declare module "*.gif" {
  const url: string
  export default url
}

declare module "*.jpg" {
  const url: string
  export default url
}

declare module "*.jpeg" {
  const url: string
  export default url
}

declare module "*.png" {
  const url: string
  export default url
}

declare module "*.webp" {
  const url: string
  export default url
}

declare module "*.avif" {
  const url: string
  export default url
}

/**
 *  @file Modules for correctly typing imported files.
 */

// -- SPECIAL

declare module "*.css"

// -- VITE

declare module "*?url" {
  const text: string
  export default text
}

declare module "*?bundled-worker" {
  const text: string
  export default text
}

// -- URL REFERENCES

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

// -- STRING IMPORTS

declare module "*.yaml" {
  const contents: string
  export default string
}

/**
 *  @file Modules for correctly typing imported files.
 */

// -- SPECIAL

declare module "*.css"

// -- URL REFERENCES

declare module "*.worker.ts" {
  const ref: string
  export default ref
}

declare module "*.wasm" {
  const ref: string
  export default ref
}

declare module "*.svg" {
  const ref: string
  export default ref
}

declare module "*.bmp" {
  const ref: string
  export default ref
}

declare module "*.gif" {
  const ref: string
  export default ref
}

declare module "*.jpg" {
  const ref: string
  export default ref
}

declare module "*.jpeg" {
  const ref: string
  export default ref
}

declare module "*.png" {
  const ref: string
  export default ref
}

declare module "*.webp" {
  const ref: string
  export default ref
}

declare module "*.avif" {
  const ref: string
  export default ref
}

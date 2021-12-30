// Compiled to JSON

declare module "*.toml" {
  const json: JSONObject
  export default json
}

declare module "*.yaml" {
  const json: JSONObject
  export default json
}

declare module "*.yml" {
  const json: JSONObject
  export default json
}

// Hunspell (urls)

declare module "*.aff" {
  const url: string
  export default url
}

declare module "*.dic" {
  const url: string
  export default url
}

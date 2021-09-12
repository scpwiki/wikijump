/** Modules for correctly typing imported files. */

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

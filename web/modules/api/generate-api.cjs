const { printer } = require("@spec2ts/core")
const { generateClient } = require("@spec2ts/openapi-client")
const fs = require("fs/promises")
const yaml = require("js-yaml")

async function getSpec(path) {
  let str = await fs.readFile(path, "utf8")
  // due to a bug in spec2ts, we're gonna replace `4XX` with `400`
  str = str.replaceAll(/4XX:/g, "400:")
  return yaml.load(str)
}

async function generate(path) {
  const spec = await getSpec(path)
  const { client, types } = await generateClient(spec, {
    inlineRequired: true,
    typesPath: "./types"
  })
  const clientString = processClient(printer.printFile(client))
  const typesString = printer.printFile(types)

  await fs.writeFile("./vendor/output.ts", clientString)
  await fs.writeFile("./vendor/types.ts", typesString)
}

function processClient(str) {
  // fix the type import
  str = str.replace("import {", "import type {")
  // fix the fetch function for void api calls
  str = str.replace(
    "fetch(url: string, req?: FetchRequestOptions): Promise<ApiResponse<string | undefined>>",
    "fetch(url: string, req?: FetchRequestOptions): Promise<ApiResponse<any>>"
  )
  // turn the exported functions into a class
  str = str.replace(
    "export type ApiResult<Fn> = Fn extends (...args: any) => Promise<ApiResponse<infer T>> ? T : never;",
    "export type ApiResult<Fn> = Fn extends (...args: any) => Promise<ApiResponse<infer T>> ? T : never;\nexport class API {"
  )
  str += "}\n"
  str = str.replaceAll("export async function", "async")
  // change functions so that they unwrap the ApiResponse
  str = str.replace("export const _ = {", `export const _ = {${unwrapResponse}`)
  str = str.replaceAll(
    /options\?: RequestOptions\): Promise<ApiResponse<(.+)>>/g,
    "options?: RequestOptions): Promise<$1>"
  )
  str = str.replaceAll("return await http", "return await _.unwrap(http")
  str = str.replaceAll(/^(    \}\)?\));/gm, "$1);")
  return str
}

const unwrapResponse = `
    unwrap<T>(res: Promise<ApiResponse<T>>): Promise<T> {
      return res.then(r => {
        if (r.data !== undefined) return r.data
      }) as Promise<T>
    },`

generate("../../resources/api/api.oas3.yaml")

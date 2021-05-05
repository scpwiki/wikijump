import * as fse from "fs-extra" // doesn't have full ESM support yet
import * as fs from "fs/promises"
import * as path from "path"
import * as tjs from "typescript-json-schema"
import ora from "ora"
import svelte2tsx from "svelte2tsx"
import globby from "globby"

// TS isn't a peer dependency for `typescript-json-schema`, which is unfortunate
// that means it'll download its own version, which means we have to use that instead
// this is probably fragile - so be careful updating `typescript-json-schema`
import ts from "typescript-json-schema/node_modules/typescript/lib/typescript.js"

// apparently this follows browser conventions in node
const DIR = path
  .dirname(import.meta.url)
  .replace("file:///", process.platform === "win32" ? "" : "/")

// make sure we're working in the right directory
process.chdir(DIR)

/** @type ts.CompilerOptions */
const TS_CONFIG = {
  moduleResolution: ts.ModuleResolutionKind.NodeJs,
  skipLibCheck: true,
  skipDefaultLibCheck: true,
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  declaration: true,
  emitDeclarationOnly: true,
  strict: true,
  outDir: "dist"
}

/** @type Partial<tjs.Args> */
const TJS_CONFIG = {
  ignoreErrors: true,
  aliasRef: true,
  excludePrivate: true,
  strictNullChecks: true
}

build()

async function build() {
  const spinner = ora({
    prefixText: "[components-docs]",
    text: "Finding components..."
  }).start()

  const msg = text => {
    spinner.text = `${text}\n`
    spinner.render()
  }

  const files = await globby("src/**/*.svelte", { absolute: true })

  msg("Generating declaration sources...")

  const map = new Map()
  for (const file of files) {
    map.set(`${file}.tsx`, await generateTSX(file))
  }

  msg("Generating declarations...")

  const { program, tsxFiles } = await createProgram(files, map)

  msg("Writing typedoc declarations...")

  program.emit()

  msg("Generating schema...")

  const gen = tjs.buildGenerator(program, TJS_CONFIG, tsxFiles)
  const symbols = gen.getUserSymbols()
  const schema = gen.getSchemaForSymbols(symbols, false)

  msg("Writing JSON...")

  const outfile = JSON.stringify(schema, undefined, 2)
    .replaceAll('"$ref":', '"ref":')
    .replaceAll('"#/definitions/', '"')
    .replaceAll('"global.', '"')
    .replaceAll(/\{@link\s+(\S+?)\s*\}/g, "`$1`")
    .replaceAll(/_\d+"/g, '"')

  await fse.outputFile("dist/docs.json", outfile)

  spinner.succeed("Components documentation JSON generated.")
}

// credit to:
// https://github.com/material-svelte/material-svelte/blob/main/tools/svelte-type-generator/src/index.ts
// for figuring out how to do this.
// I use the exact same method here, but with different post-processing

async function createProgram(files, map) {
  const host = ts.createCompilerHost(TS_CONFIG)

  const fileExists = host.fileExists
  host.fileExists = filename => {
    if (filename.endsWith(".svelte.tsx")) return true
    return fileExists(filename)
  }

  const readFile = host.readFile
  host.readFile = filename => {
    if (filename.endsWith(".svelte.tsx")) {
      return map.get(filename)
    }
    return readFile(filename)
  }

  const writeFile = host.writeFile
  host.writeFile = (filename, data, BOM, onErr, sourceFiles) => {
    if (filename.endsWith(".svelte.d.ts")) {
      // this is a hack, but I couldn't get anything else to cooperate
      // this makes it so that the props, events, and slots
      // properties are not optional, which screws up types otherwise
      data = data
        .replaceAll("    props?: {", "    props: {")
        .replaceAll("    events?: ", "    events: ")
        .replaceAll("    slots?: ", "    slots: ")
        .replaceAll("    } | undefined;", "    };")
        .replaceAll(" {} | undefined;", " {};")
    }
    return writeFile(filename, data, BOM, onErr, sourceFiles)
  }

  const tsxFiles = files.map(file => `${file}.tsx`)
  const shims = "../../node_modules/svelte2tsx/svelte-shims.d.ts"
  const types = await globby("../misc/types/*.d.ts", { absolute: true })

  const program = ts.createProgram([...tsxFiles, shims, ...types], TS_CONFIG, host)

  return { program, tsxFiles }
}

// prettier-ignore
const COMPONENT_REGEX =
 /export default class (.+)__SvelteComponent_ extends createSvelte2TsxComponent\((.+)\)\s*\{\s*\}/

async function generateTSX(file) {
  const src = await fs.readFile(file, "utf-8")
  const tsx = svelte2tsx(src, { filename: file, strictMode: true, isTsFile: true })

  // postprocessing
  const match = COMPONENT_REGEX.exec(tsx.code)
  if (match) {
    const [oldExport, name, props] = match

    const newExport =
      `export declare class ${name} {` +
      `\n  constructor(options: IComponentOptions)` +
      `\n  props: typeof props.props;` +
      `\n  events: typeof props.events;` +
      `\n  slots: typeof props.slots;` +
      `\n}`

    // prettier-ignore
    const out =
      `\nexport interface IComponentOptions { target: Element; anchor?: Element; props?: Record<string, any>; context?: Map<any, any>; hydrate?: boolean; intro?: boolean; };` +
      `\n${tsx.code.replace(oldExport, newExport)};` +
      `\nconst props = ${props}();`

    return out
  }

  throw new Error(`Failed to generate TSX for file: ${file}`)
}

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
import ts from "../node_modules/typescript-json-schema/node_modules/typescript/lib/typescript.js"

// apparently this follows browser conventions in node
const DIR = path.dirname(import.meta.url).replace("file:///", "")

/** @type ts.CompilerOptions */
const TS_CONFIG = {
  moduleResolution: ts.ModuleResolutionKind.NodeJs,
  skipLibCheck: true,
  skipDefaultLibCheck: true,
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  noEmit: true
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
    spinner.text = text
    spinner.render()
  }

  const files = await globby("src/**/*.svelte", { cwd: DIR, absolute: true })

  msg("Generating declaration sources...")

  const map = new Map()
  for (const file of files) {
    map.set(`${file}.tsx`, await generateTSX(file))
  }

  msg("Generating declarations...")

  const { program, tsxFiles } = await createProgram(files, map)

  msg("Generating schema...")

  const gen = tjs.buildGenerator(program, TJS_CONFIG, tsxFiles)
  const symbols = gen.getUserSymbols()
  const schema = gen.getSchemaForSymbols(symbols, false)

  msg("Emitting JSON...")

  const outfile = JSON.stringify(schema, undefined, 2)
    .replaceAll('"$ref":', '"ref":')
    .replaceAll('"#/definitions/', '"')
    .replaceAll('"global.', '"')
    .replaceAll(/\{@link\s+(\S+?)\s*\}/g, "`$1`")

  await fse.outputFile(path.resolve(DIR, "./dist/docs.json"), outfile)

  spinner.succeed("Components documentation JSON generated.")
}

// credit to:
// https://github.com/material-svelte/material-svelte/blob/main/tools/svelte-type-generator/src/index.ts
// for figuring out how to do this.
// I use the exact same method here, but with different post-processing

async function createProgram(files, map) {
  const host = ts.createCompilerHost(TS_CONFIG)

  const originalFileExists = host.fileExists
  host.fileExists = filename => {
    if (filename.endsWith(".svelte.tsx")) return true
    return originalFileExists(filename)
  }

  const originalReadFile = host.readFile
  host.readFile = filename => {
    if (filename.endsWith(".svelte.tsx")) {
      return map.get(filename)
    }
    return originalReadFile(filename)
  }

  const tsxFiles = files.map(file => `${file}.tsx`)
  const shims = path.resolve(DIR, "../node_modules/svelte2tsx/svelte-shims.d.ts")
  const types = await globby("../misc/types/*.d.ts", { cwd: DIR, absolute: true })

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
      `\n  props: typeof props.props;` +
      `\n  events: typeof props.events;` +
      `\n  slots: typeof props.slots;` +
      `\n}`

    const out =
      `import { SvelteComponentTyped } from 'svelte';` +
      `\n${tsx.code.replace(oldExport, newExport)}` +
      `\nconst props = ${props}();`

    return out
  }

  throw new Error(`Failed to generate TSX for file: ${file}`)
}

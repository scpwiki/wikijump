import path from "path"
// this import is why this script has to be ran with `esbuild-runner`
import { DevCLI } from "./dev/cli.ts"

const DIR = path.resolve(__dirname, "../")
process.chdir(DIR)

DevCLI.create()

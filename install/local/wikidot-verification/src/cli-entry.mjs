import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";

export function isDirectExecution(moduleUrl, argv = process.argv) {
  return Boolean(argv[1]) && path.resolve(argv[1]) === fileURLToPath(moduleUrl);
}

export async function runCliIfMain(moduleUrl, main, {
  argv = process.argv.slice(2),
  processArgv = process.argv,
  onError = (error) => {
    console.error(error?.stack ?? error?.message ?? String(error));
    return 1;
  },
} = {}) {
  if (!isDirectExecution(moduleUrl, processArgv)) return false;
  try {
    const code = await main(argv);
    if (Number.isInteger(code)) process.exitCode = code;
  } catch (error) {
    process.exitCode = onError(error);
  }
  return true;
}

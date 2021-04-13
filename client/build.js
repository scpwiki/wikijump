const { build, cliopts, basename, stdoutStyle: styl, fmtDuration } = require('estrella')
const { nodeExternalsPlugin } = require('esbuild-node-externals')
const { readdirSync, realpathSync } = require('fs')
const { performance } = require('perf_hooks')

const dev = !!cliopts.watch
const [opts] = cliopts.parse(['self', 'Compiles the calling package.'])

const SETTINGS_COMMON = {
  // esbuild options
  outdir: 'dist',
  bundle: true,
  treeShaking: true,
  splitting: true,
  platform: 'browser',
  format: 'esm',
  sourcemap: true,
  sourcesContent: true,
  // estrella options
  tslint: false,
  silent: true
}

function getDirectories(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name)
}

function buildPackage(name) {
  // figure out our paths based on if the package is self-compiling or not
  const self = !name // undefined or empty
  const dir = self ? process.cwd() : realpathSync('./packages/' + name)
  const index = (self ? '.' : dir) + '/src/index.ts'
  const package = (self ? '.' : dir) + '/package.json'

  name = basename(dir)

  if (!dev) console.log(styl.blue(`[${name}]`), 'Building!')
  let start = performance.now()

  // build the darn thing
  build({
    ...SETTINGS_COMMON,
    absWorkingDir: dir,
    entry: [index],
    tsconfig: package,
    plugins: [nodeExternalsPlugin({ packagePath: package })],
    cwd: dir,
    // estrella uses Chokidar, so this property uses the Chokidar watch options interface
    watch: dev && {
      cwd: dir
    },
    // logging handlers
    onStart(_, changed) {
      if (dev && changed && changed.length) {
        start = performance.now()
        console.log(styl.blue(`[${name}]`), 'Rebuilding!', styl.orange(`[${changed.join(', ')}]`))
      }
    },
    onEnd() {
      const elapsed = fmtDuration(performance.now() - start)
      console.log(styl.blue(`[${name}]`), 'Finished.', styl.green(`${elapsed}`))
    }
  })
}

// check if we're building all or just a single package
if (opts.self) {
  buildPackage()
} else {
  getDirectories('./packages').forEach(name => buildPackage(name))
}

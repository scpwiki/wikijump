const { build, cliopts, basename, stdoutStyle: styl, fmtDuration } = require('estrella')
const { nodeExternalsPlugin } = require('esbuild-node-externals')
const { readdirSync, realpathSync } = require('fs')
const { performance } = require('perf_hooks')

const dev = !!cliopts.watch
const [opts] = cliopts.parse(['self', 'Compiles the calling module package only.'])

const SETTINGS_COMMON = {
  // esbuild settings
  outdir: 'dist',
  // minify: true // estrella implicitly toggles minify depending on the `--debug` flag
  bundle: true,
  treeShaking: true,
  splitting: true,
  platform: 'browser',
  format: 'esm',
  sourcemap: true,
  sourcesContent: true,

  // estrella settings
  tslint: false, // disables estrella's built-in typechecker
  silent: true   // silences estrella's logging, as we use our own logging
}

function directoriesOf(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name)
}

function buildModule(name) {
  const self = !name // undefined or empty

  let dir, index, package

  if (self) {
    dir = process.cwd()
    index = './src/index.ts'
    package = './package.json'
  } else {
    dir = realpathSync('./modules/' + name)
    index = dir + '/src/index.ts'
    package = dir + '/package.json'
  }

  name = basename(dir)

  if (!dev) console.log(styl.blue(`[${name}]`), 'Building!')
  let start = performance.now()

  build({
    ...SETTINGS_COMMON,

    // esbuild settings
    absWorkingDir: dir,
    tsconfig: package,
    plugins: [nodeExternalsPlugin({ packagePath: package })],

    // estrella settings
    entry: index,
    cwd: dir,
    // estrella uses Chokidar, so this property uses the Chokidar watch settings interface
    // we need to make sure that Chokidar is watching the module directory
    watch: dev && { cwd: dir },

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
if (opts.self) buildModule()
else directoriesOf('./modules').forEach(name => buildModule(name))

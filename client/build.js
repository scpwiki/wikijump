const { build, cliopts } = require('estrella')
const { nodeExternalsPlugin } = require('esbuild-node-externals')
const { readdirSync, realpathSync } = require('fs')

const [opts] = cliopts.parse(['self', 'Compiles the calling package.'])

const SETTINGS_COMMON = {
  outdir: 'dist',
  bundle: true,
  treeShaking: true,
  splitting: true,
  platform: 'browser',
  format: 'esm',
  sourcemap: true,
  sourcesContent: true,
  // estrella
  tslint: false
}

function getDirectories(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name)
}

function buildPackage(name) {
  // figure out our paths based on if the package is self-compiling or not
  const self = name === '.'
  const dir = self ? process.cwd() : realpathSync('./packages/' + name)
  const index = (self ? '.' : dir) + '/src/index.ts'
  const package = (self ? '.' : dir) + '/package.json'
  // build the darn thing
  build({
    ...SETTINGS_COMMON,
    absWorkingDir: dir,
    entry: [index],
    tsconfig: package,
    plugins: [nodeExternalsPlugin({ packagePath: package })],
    cwd: dir,
    // estrella uses Chokidar, so this property uses the Chokidar watch options interface
    watch: cliopts.watch && {
      cwd: dir
    }
  })
}

// check if we're doing building all or just a single package
if (opts.self) {
  buildPackage('.')
} else {
  getDirectories('./packages').forEach(name => buildPackage(name))
}

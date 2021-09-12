/*
  This file is how PNPM lets you hook into its install process.
  Right now, this is being used to fix a really annoying type validation issue with CodeMirror.
*/

// TODO: remove these when possible, as otherwise these deps can't be updated!

function readPackage(pkg) {
  fixDependency(pkg, "@codemirror/view", "^0.19.6")
  fixDependency(pkg, "@lezer/common", "^0.15.5")
  return pkg
}

function fixDependency(pkg, dependency, version) {
  if (pkg?.dependencies?.[dependency] && pkg.dependencies[dependency] !== version) {
    pkg.dependencies[dependency] = version
    console.log(`Replaced ${dependency} version in package ${pkg.name ?? "(unknown)"}`)
  }
}

module.exports = { hooks: { readPackage } }

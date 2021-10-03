/**
 * This file is a special "test" that imports, quite literally, every
 * single module through a Vite import glob. This is so that code coverage
 * is checked even for modules for which there are no tests.
 */

const EveryModule = import.meta.globEager("../modules/*/src/index.ts")

export {}

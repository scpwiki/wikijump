# ftml-wasm

This package contains the WebAssembly compilation of FTML.

For the time being, this WASM compilation is built statically and committed, for the following reasons:

- A frontend developer does not need to install Rust to get quickly set up with an inital build.
- The current build process is not equipped to handle external files e.g. FTML.
- Breaking changes to FTML will not break the bindings in this package - they will simply be outdated.
- The full build process is much faster, which is important during this intial phase of iterative development.
- The build process is simplified by not needing a full Rust compilation.

Eventually, the WASM compilation will be run during a build step and will be up-to-date with the current FTML version.

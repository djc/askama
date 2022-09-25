# Building the documentation

## Setup

1. Install MDBook - `cargo install mdbook`
2. Run `./build.sh` to build all versions
3. View in your browser: `open dist/index.html`

## Organization

* **Prior releases** -- folders such as `0.11.x/` or `0.10.x/` contains docs for prior releases
* **Current stable version** -- the `index.html` file contains a redirect to the docs for current stable release
* **Unreleased content** -- docs for new, unreleased features go into `next/`

## Releasing a new version

Run `./prepare_release.sh <CURRENT_VERSION> <NEXT_VERSION>`. 

CURRENT_VERSION = the current 'stable' version of the library
NEXT_VERSION = the version of the library you are preparing to release

This takes a current snapshot of `next/` and makes that into the documentation.

Example: `./prepare_release.sh 0.11.x 0.12.x`

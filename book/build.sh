#!/bin/bash

#
# Compiles documentation for all versions
#

set -eu

mkdir -p dist
rm -rf ./dist/*

# Loop through all directories that aren't `common` or `dist`
for d in */ ; do
    if [[ $d == "common/" || $d == "dist/" ]]; then
        continue
    else
        cd $d
        mdbook build -d ../dist/$d
        cd ../
    fi
done

# Copy in the redirection for the latest stable version
cp index.html dist/index.html
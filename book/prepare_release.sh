#!/bin/bash


#
# Prepares for a new release of the library
#

set -eu

current_version="${1:-}"
next_version="${2:-}"

if [[ -z $current_version ]]; then
    echo "usage: $0 <CURRENT_VERSION> <NEXT_VERSION>"
    echo "error: missing argument <CURRENT_VERSION>"
    exit 1
fi
if [[ -z $next_version ]]; then
    echo "usage: $0 <CURRENT_VERSION> <NEXT_VERSION>"
    echo "error: missing argument <NEXT_VERSION>"
    exit 1
fi

if [[ -d $next_version ]]; then
    echo "error: directory $next_version/ already exists"
    exit 1
fi

set -x # show all commands 

mkdir -p $current_version/theme
ln -s ../../common/header_outdated.hbs $current_version/theme/header.hbs
cp -r next $next_version
rm -r $next_version/theme
cat << EOD > $next_version/book.toml
[book]
authors = ["djc", "cetra3"]
language = "en"
multilingual = false
src = "src"
title = "Askama ($next_version)"
EOD

cat << EOD > index.html
<!-- This file redirects the user to the latest stable version. -->
<!DOCTYPE html>
<html>
    <head>
        <meta
            http-equiv="refresh"
            content="0; url = ./$next_version/index.html"
        />
    </head>
</html>
EOD
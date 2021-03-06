#!/bin/sh

# Prerequisites:
#     cargo install svgcleaner

set -e
cd "$(dirname "$(readlink -f "$0")")/../src"

find . -name "*.svg" -exec sh -c  "echo {}; svgcleaner '{}' '{}' --simplify-transforms --paths-coordinates-precision 2 --properties-precision 2" \;

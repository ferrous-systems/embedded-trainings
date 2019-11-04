#!/usr/bin/env bash

set -o errexit

for path in advanced beginner beginner-lite; do
    (
        echo "=> Building $path/templates ..."
        cd "$path/templates"
        cargo build
    )

    if [ -d "$path/utilities" ]; then
        for util in "$path"/utilities/*; do
            (
                echo "=> Building $util ..."
                cd "$util"
                cargo build
            )
        done
    fi
done

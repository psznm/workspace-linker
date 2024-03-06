#!/bin/bash

if [ "$(git status --porcelain)" != "" ]; then
	echo "Clean repository first";
	exit 1;
fi
cargo build -r
npm version patch
npm publish



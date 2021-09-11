#! /bin/sh
set -x
if cargo run; then
  ./output
  echo "Exit Code: $?"
fi

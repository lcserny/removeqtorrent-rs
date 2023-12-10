#!/bin/bash

DEST_DIR="/home/leonardo/bin/videosmover/remove-qtorrent"

choose_dest_dir () {
    echo "Enter removeqtorrent destination directory or leave blank to use default:"
    echo "Default: ${DEST_DIR}"

    read USER_INPUT

    if test "$USER_INPUT" 
    then
        DEST_DIR="$USER_INPUT"
    fi
}

choose_dest_dir

echo ""
echo "Running tests"
cargo nextest run
if [ $? -ne 0 ]; then
  exit 1
fi

echo ""
echo "Building removeqtorrent for release"
cargo build --release
if [ $? -ne 0 ]; then
  exit 1
fi

echo "Installing removeqtorrent to target destination"
cp target/release/removeqtorrent "$DEST_DIR/removeqtorrent"
if [ $? -ne 0 ]; then
  exit 1
fi

echo ""
echo "Done!"
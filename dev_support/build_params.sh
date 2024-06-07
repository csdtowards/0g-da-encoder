#!/bin/bash

# Check if the number of arguments is 1
if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <challenge-path>"
  exit 1
fi

# Get the origin-path argument
ORIGIN_PATH="$1"

# Check if origin-path exists
if [ ! -e "$ORIGIN_PATH" ]; then
  echo "Error: $ORIGIN_PATH does not exist."
  exit 1
fi

# Get the filename from origin-path
LINK_NAME="challenge_28"

rm "$LINK_NAME"
# Create a symbolic link
ln -s "$ORIGIN_PATH" "$LINK_NAME"
echo "Created symbolic link: $LINK_NAME -> $ORIGIN_PATH"

# Build params
mkdir params
cargo run -r -p ppot2ark --features ppot2ark/parallel -- . 28 20 ./params && 
cargo run -r -p amt --features amt/parallel,amt/cuda-bn254 --bin build_params -- 20 10 3 ./params

# Check the command execution result
if [ $? -ne 0 ]; then
  echo "Error: Command execution failed."
  rm "$LINK_NAME"
  echo "Removed symbolic link: $LINK_NAME"
  exit 1
fi

# Remove the symbolic link
rm "$LINK_NAME"
echo "Removed symbolic link: $LINK_NAME"

exit 0

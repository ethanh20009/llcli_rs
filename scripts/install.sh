#!/bin/bash

# Define the installation directory
INSTALL_DIR="/usr/local/bin"

echo "Building project in release mode..."
# Build the project in release mode
cargo build --release
# Check if the build was successful
if [ $? -ne 0 ]; then
  echo "Error: cargo build failed."
  exit 1
fi

echo "Finding compiled executable..."
# Find the main executable in target/release.
# This command looks for an executable file in the target/release directory,
# excludes files ending in '.d' (which are dependency files), and picks the first one found.
EXECUTABLE=$(find target/release/ -maxdepth 1 -type f -executable ! -name "*.d" | head -n 1)

# Check if an executable was found
if [ -z "$EXECUTABLE" ]; then
  echo "Error: Could not find the executable in target/release/."
  echo "Please ensure your Cargo.toml defines a binary target."
  exit 1
fi

# Get the name of the executable file
EXECUTABLE_NAME=$(basename "$EXECUTABLE")

echo "Copying '$EXECUTABLE_NAME' to '$INSTALL_DIR/'..."
# Use sudo to copy the executable to the system-wide installation directory
# You will be prompted for your password
sudo cp "$EXECUTABLE" "$INSTALL_DIR/"

# Check if the copy operation was successful
if [ $? -ne 0 ]; then
  echo "Error: Failed to copy the executable to '$INSTALL_DIR/'."
  echo "Please ensure you have the necessary permissions (sudo)."
  exit 1
fi

echo "Successfully installed '$EXECUTABLE_NAME' to '$INSTALL_DIR/'."

exit 0

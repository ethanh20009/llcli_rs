#!/bin/bash

# Define the installation directory
INSTALL_DIR="/usr/local/bin"

# Parse arguments
SHELL_ARG=""
while [[ "$#" -gt 0 ]]; do
  case $1 in
  --shell)
    SHELL_ARG="$2"
    shift # past argument
    shift # past value
    ;;
  *)
    echo "Unknown argument: $1"
    exit 1
    ;;
  esac
done

# Handle possible terminal integrations
if [ -z "$SHELL_ARG" ]; then
  SHELL_ARG="none"
fi

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

echo "Checking for shell integration options."

if [[ "$SHELL_ARG" == "none" ]]; then
  echo "No shell integration will be used."
# elif [[ "$SHELL_ARG" == "zsh" ]]; then
#   echo "Zsh shell integration will be used."
elif [[ "$SHELL_ARG" == "fish" ]]; then
  echo "Fish shell integration will be used."
  # Check fish installed
  if ! command -v fish &>/dev/null; then
    echo "Fish shell is not installed. Please install it first."
    exit 1
  fi
  # Use integrations/install_fish.fish relative from this script.
  INSTALL_FISH_SCRIPT="$(dirname "$0")/integrations/install_fish.fish"
  fish "$INSTALL_FISH_SCRIPT"
else
  echo "Unknown shell argument: $SHELL_ARG"
  exit 1
fi

echo "Successfully installed '$EXECUTABLE_NAME' to '$INSTALL_DIR/'."

exit 0

#!/usr/bin/env fish

# Copy llcli_rs/integrations/fish into ~/.config/fish

set INSTALL_DIR "$HOME/.local/bin"
set FISH_CONFIG_DIR "$HOME/.config/fish"

# Check if the fish config directory exists
if not test -d "$FISH_CONFIG_DIR"
    echo "Error: Fish config directory '$FISH_CONFIG_DIR' does not exist."
    echo "Please ensure you have Fish shell installed and configured."
    exit 1
end

# Check if the fish config directory is writable
if not test -w "$FISH_CONFIG_DIR"
    echo "Error: Fish config directory '$FISH_CONFIG_DIR' is not writable."
    echo "Please check your permissions."
    exit 1
end

set ROOT_PROJECT_DIR (dirname (dirname (dirname (status --current-filename))))

echo "Copying Fish integration files to '$FISH_CONFIG_DIR'..."

cp "$ROOT_PROJECT_DIR/integrations/fish/conf.d/llcli_rs.fish" "$FISH_CONFIG_DIR/conf.d/"

# Recursive copy all functions
set FISH_FUNC_DIR "$FISH_CONFIG_DIR/functions"
if not test -d "$FISH_FUNC_DIR"
    mkdir -p "$FISH_FUNC_DIR"
end
cp -r "$ROOT_PROJECT_DIR/integrations/fish/functions/"* "$FISH_FUNC_DIR/"

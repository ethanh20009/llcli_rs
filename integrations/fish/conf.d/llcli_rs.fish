if test -n "$LLCLI_RS_KEYMAP_1"
    set -g keymap_1 "$LLCLI_RS_KEYMAP_1"
else
    set -g keymap_1 ctrl-space
end
bind $keymap_1 _llcli_rs_gencommand

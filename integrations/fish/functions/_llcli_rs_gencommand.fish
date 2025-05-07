function _llcli_rs_gencommand --description "Generate a command using llcli_rs"
    set -l prompt (commandline --current-buffer)

    # Generate the command
    set -l generated_command (llcli_rs -q code -m "User is using Fish shell. $prompt")

    # Strip trailing newline
    set -l generated_command (string trim $generated_command)

    # replace input with the generated command
    commandline --replace $generated_command

    # Set cursor to the end of the line
    set current_pos (commandline --cursor)
    set command_length (string length -V $generated_command)
    set command_length_total 0
    for item_length in $command_length
        if test $item_length != ""
            set command_length_total (math "$command_length_total + $item_length")
        end
    end
    set new_pos (math "$current_pos + $command_length_total")
    commandline --cursor --replace $new_pos
end

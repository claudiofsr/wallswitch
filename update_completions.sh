#!/usr/bin/env bash

# wallswitch --generate=bash > completions/completion_derive.bash
# wallswitch --generate=elvish > completions/completion_derive.elvish
# wallswitch --generate=fish > completions/completion_derive.fish
# wallswitch --generate=powershell > completions/completion_derive.powershell
# wallswitch --generate=zsh > completions/completion_derive.zsh

shells="bash elvish fish powershell zsh"

for shell in $shells; do
 wallswitch --generate=$shell > completions/completion_derive.$shell
done

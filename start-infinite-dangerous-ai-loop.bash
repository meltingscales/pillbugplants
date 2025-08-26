echo "Starting infinite claude loop. You may not see output for a while. CTRL-C will not kill this, you need to kill the terminal."
while :; do cat PROMPT.md | claude -p --dangerously-skip-permissions; done

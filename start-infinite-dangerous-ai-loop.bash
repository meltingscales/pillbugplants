echo "Starting infinite AI loop with devstral small 1.1 at 192.168.0.186. You may not see output for a while. CTRL-C will not kill this, you need to kill the terminal."
export OLLAMA_API_BASE=http://192.168.0.186:11434
poetry install
poetry run aider-install
while :; poetry run aider --message-file PROMPT.md --model ollama_chat/llama3.2:3b --restore-chat-history --yes-always --no-fancy-input; done


# todo - ask ai below to fix this... and make sure aiders source code is open in its context.
:<<END_COMMENT
I'd like to use aider the same way as claude CLI, i.e.

while :; do cat prompt.md | claude -p --dangerously-skip-permissions; done

This claude call will run claude forever, editing files and doing whatever prompt.md instructs it to, essentially turning it into an immortal software programmer.

However, aider doesn't automatically edit files and do other stuff like claude does. I'd like you to analyze Aider's codebase, which is open, and find out how I can get the same behavior. This is what I have so far, and it loads PROMPT.md but doesn't make code edits the same way:

echo "Starting infinite AI loop with devstral small 1.1 at 192.168.0.186. You may not see output for a while. CTRL-C will not kill this, you need to kill the terminal."
export OLLAMA_API_BASE=http://192.168.0.186:11434
poetry install
poetry run aider-install
while :; poetry run aider --model ollama_chat/llama3.2:3b PROMPT.md; done
END_COMMENT

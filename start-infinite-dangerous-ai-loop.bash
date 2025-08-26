echo "Starting infinite AI loop with devstral small 1.1 at 192.168.0.186. You may not see output for a while. CTRL-C will not kill this, you need to kill the terminal."
export OLLAMA_API_BASE=http://192.168.0.186:11434
poetry install
poetry run aider-install
while :; poetry run aider --model ollama_chat/llama3.2:3b PROMPT.md; done

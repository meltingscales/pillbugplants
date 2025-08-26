echo "Starting infinite AI loop with devstral small 1.1 at 192.168.0.186. You may not see output for a while. CTRL-C will not kill this, you need to kill the terminal."
export OLLAMA_HOST=http://192.168.0.186:11434
poetry install
poetry run llm install llm-ollama
poetry run llm ollama models
while :; do cat PROMPT.md | poetry run llm -m devstral-small-1.1; done

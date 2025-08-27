
if [ ! -f "./setup-env.bash" ]; then
    echo "Please populate setup-env.bash with env vars that are needed."
    exit 1
fi


# Check if the VIRTUAL_ENV variable is set
if [ -z "$VIRTUAL_ENV" ]; then
  echo "Not in a virtual environment. Exiting. Please run this within 'poetry shell'"
  exit 1 # Exit with a non-zero status to indicate an error
fi

source ./setup-env.bash

echo "Starting infinite AI loop with $MODEL at $OLLAMA_API_BASE. You may not see output for a while."
echo "CTRL-C will not kill this, you need to kill the terminal."

poetry install
aider-install
while :; do
    aider --message-file PROMPT.md --model ollama_chat/$MODEL --restore-chat-history --yes-always --no-fancy-input
done

#  --max-reflections 99999 
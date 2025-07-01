# LiteLLM for Gemini CLI

This is to test something that isn't working at work.

It fails in a similar way as of 2025-06-28 and LiteLLM version 1.73.6: failure to parse JSON tool call responses from the model.

This seems to be a problem with LiteLLM, since using the Google API directly works fine.

## Running LiteLLM
This uses an API key stored in 1Password.

```shell
pip install 'litellm[proxy]'
pip install google-genai
export GEMINI_API_KEY=$(op item get vg55xz2n6pfdkhrljiupfrlpfq --fields "credential" --reveal)
litellm --config config.yaml --detailed_debug
```

## Running Gemini CLI

You may need to delete or move ~/.gemini to ensure that cached credentials and settings are not used.

```shell
export GOOGLE_GEMINI_BASE_URL=http://localhost:4000/
export GEMINI_API_KEY=sk-1234567890
gemini
```

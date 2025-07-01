# LiteLLM for Gemini CLI
This is to test something that isn't working at work.

It fails in a similar way as of 2025-06-28 and LiteLLM version 1.73.6: failure to parse JSON tool call responses from the model.

This seems to be a problem with LiteLLM, since using the Google API directly works fine.

## Running LiteLLM
This uses an API key stored in 1Password.

```shell
export GEMINI_API_KEY=$(op item get vg55xz2n6pfdkhrljiupfrlpfq --fields "credential" --reveal)
./start.sh
```

## Running Gemini CLI
You may need to delete or move ~/.gemini to ensure that cached credentials and settings are not used.

```shell
export GOOGLE_GEMINI_BASE_URL=http://localhost:4000/
export GEMINI_API_KEY=sk-1234567890
gemini
```

## Gemini Errors
Here is the error in the Gemini CLI console:
```
 Debug Console (ctrl+o to close)

 ℹ  Authenticated via "gemini-api-key".
 ⚠  Could not determine token count for model gemini-2.5-pro. Skipping compression check.
 ✖  Failed to parse JSON response from generateJson. Full report available at:
    /var/folders/cp/zjqv523x0tb03yr78gxyfwk40000gn/T/gemini-client-error-generateJson-parse-2025-07-01T16-15-56-535Z.j
    son
 ✖  Error generating JSON content via API. Full report available at:
    /var/folders/cp/zjqv523x0tb03yr78gxyfwk40000gn/T/gemini-client-error-generateJson-api-2025-07-01T16-15-56-537Z.jso
    n
 ⚠  Failed to talk to Gemini endpoint when seeing if conversation should continue. Error: Failed to generate JSON
    content: Failed to parse API response as JSON: Unexpected token '`', "```json
    {
    "... is not valid JSON
        at GeminiClient.generateJson
    (file:///Users/mch/.local/share/mise/installs/node/24.2.0/lib/node_modules/@google/gemini-cli/node_modules/@google
    /gemini-cli-core/dist/src/core/client.js:239:19)
```

The referenced log files are:
- [gemini-client-error-generateJson-api-2025-07-01T16-15-56-537Z.json](./gemini-client-error-generateJson-api-2025-07-01T16-15-56-537Z.json)
- [gemini-client-error-generateJson-parse-2025-07-01T16-15-56-535Z.json](./gemini-client-error-generateJson-parse-2025-07-01T16-15-56-535Z.json)

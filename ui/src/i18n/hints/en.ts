import type { Hints } from "./shape";

export const hints: Hints = {
  open: "Hint",
  advice: "Recommended:",
  apply: "Fill in the recommended ones",
  own: "There is no recommendation for your own command — see its own help (--help).",
  args: [
    "One line is one whole argument: the app does not split a line on spaces.",
    "The value of a flag goes on the next line, under the flag itself.",
    "An empty line is an empty value, not a skip: without it the harness complains about a missing argument.",
    "The app sends the request text itself — do not put it into the arguments.",
  ],
  presets: {
    claude: [
      {
        lines: ["-p"],
        why: "a single run instead of an interactive session: the answer goes to stdout and the harness exits",
      },
      {
        lines: ["--output-format", "stream-json"],
        why: "the answer arrives as a stream — the screen shows it as it comes and counts the tokens",
      },
      {
        lines: ["--verbose"],
        why: "without it claude refuses to print stream-json under -p",
      },
      {
        lines: ["--include-partial-messages"],
        why: "pieces of the answer as it is written; without them the screen stays silent to the end and the silence timeout can cut a live request",
      },
      {
        lines: ["--allowedTools", ""],
        why: "no tools — the app only needs text. The empty line is required: without it claude answers «option '--allowedTools' argument missing»",
      },
    ],
    opencode: [
      {
        lines: ["run"],
        why: "a one-off run with the request on the input, no interactive mode",
      },
    ],
    pi: [
      { lines: ["-p"], why: "a one-off run, the answer goes to stdout" },
      { lines: ["--no-tools"], why: "no tools — only text is needed" },
      { lines: ["--no-session"], why: "keep no session: every request stands on its own" },
    ],
    custom: [],
  },
  local: [
    "Endpoint: for ollama it is http://127.0.0.1:11434, for an openai-compatible server it is the address with /v1 at the end.",
    "Model: the name exactly as the server has it (ollama shows it with `ollama list`).",
    "Context: 0 leaves it as the server is set up. The app needs 32k and up; at 8k the skeleton no longer builds.",
    "Temperature: 0.7 is the working value. Above 1.0 the model starts breaking the YAML and the build goes in repair rounds.",
    "A reasoning model is better run with thinking off (`--reasoning-budget 0` for llama-server): otherwise the answer takes three times longer and the YAML gains nothing.",
  ],
  remote: [
    "Endpoint: the address of an OpenAI-compatible API with /v1 at the end.",
    "Model: the name exactly as the vendor has it.",
    "The key lives in the system store, not in the config: do not put it into the endpoint or the arguments.",
    "This is the only mode in which the request text leaves the machine.",
  ],
};

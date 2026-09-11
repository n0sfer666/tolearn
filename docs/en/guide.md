# User guide

[Русский](../ru/guide.md) · **English**

The short version of how to use the app. For installation see
[install/](install/README.md).

## Getting started

1. Get a learning program. The prompts for that live in this repository:
   [`core/assets/generate-roadmap.md`](../../core/assets/generate-roadmap.md)
   produces the skeleton of a program, and
   [`core/assets/generate-topic.md`](../../core/assets/generate-topic.md) produces
   a single topic file. Paste the first one into any LLM chat, add what you want
   to learn below it, take the resulting `roadmap.yaml` and `progress.yaml`, then
   use the second prompt to build the topics one by one. The result is a bundle —
   a folder with `roadmap.yaml`, `topics/*.yaml` and `examiner.md`. Example:
   [`examples/llm-agents-base`](../../examples/llm-agents-base). There is no
   "copy the prompt" button in the UI yet.
2. On the home screen press "Choose folder" or "Choose archive". The app
   validates the bundle and shows the errors, if any.
3. Open a topic and start.

## Screens

| Screen | What for |
|---|---|
| Program | every topic by stage, statuses, entry point into a topic |
| Topic | materials, practice, questions |
| Practice | timer for the topic's timebox, survives a restart |
| Exam | questions, answers and the verdict |
| Stale | topics that changed after the program was regenerated |
| Graph | topic dependencies by layer |
| Search | across topics and materials of the open program |
| Settings | language, LLM provider, offline, history |

## The exam

Two paths, both yours:

- **Copy-paste (default).** The app assembles the prompt, you paste it into any
  chat and paste the model's answer back. No key, no account.
- **Provider.** Settings let you point at Ollama (local) or an OpenAI-compatible
  endpoint with a key. The key is kept in the OS keychain.

The verdict is written into your program's `progress.yaml`. Nothing else in the
bundle is ever written to.

## Offline

A topic's materials are downloaded once and saved whole, as a single HTML file.
After that the network is not needed: the reader shows the text from the archive.
Nothing but an explicitly requested download ever goes online.

## Where your data lives

- Programs stay wherever you put them; the app remembers the path in its
  registry.
- Settings, the program registry and the store (page archives, version history)
  live in `~/.config/tolearn` and `~/.local/share/tolearn`, identically on all
  three operating systems; exact paths and how to move them —
  [install/](install/README.md#where-your-data-lives).
- Uninstalling the app does not touch your data.
- The app keeps v2 programs in its own library, in
  `~/.local/share/tolearn/programs/`. v1 program data — the registry
  `~/.config/tolearn/registry.yaml` and, under `~/.local/share/tolearn`, the
  `unpacked/`, `history/` and `offline/` directories and the `search-*` search
  index — is neither read nor removed by the app once it moves to v2: you can
  delete it by hand.

## Export

The "Export to Markdown" button on the program screen collects the whole program
into one file — table of contents and topics. Answers and traps make it into the
file only for topics that were passed. The export never writes inside the
bundle itself.

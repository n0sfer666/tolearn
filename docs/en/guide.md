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
| Topic | materials, practice, questions, note |
| Practice | timer for the topic's timebox, survives a restart |
| Exam | questions, answers and the verdict |
| Queue | what to do now: available topics and reviews that came due |
| Review | topics whose revalidation date has arrived |
| Stale | topics that changed after the program was regenerated |
| Graph | topic dependencies by layer |
| Notes · Search | all notes and search across them |
| Statistics | hours, statuses, history |
| Settings | language, LLM provider, notes, encryption, offline, history |

## The exam

Two paths, both yours:

- **Copy-paste (default).** The app assembles the prompt, you paste it into any
  chat and paste the model's answer back. No key, no account.
- **Provider.** Settings let you point at Ollama (local) or an OpenAI-compatible
  endpoint with a key. The key is kept in the OS keychain.

The verdict is written into your program's `progress.yaml`. Nothing else in the
bundle is ever written to.

## Notes

A topic note is a plain `.md` file with a `tolearn: roadmap/topic` frontmatter.
By default notes live in the app's data directory, but settings let you point at
your own vault — an Obsidian folder, for instance: edits made outside are then
picked up.

Encryption (Settings → "Encryption") covers the **internal** directory: files
become unreadable from outside and the file names give away nothing. The price is
stated right on that screen — an external editor, picking up outside edits and
the Obsidian plugin all stop working, and losing both the passphrase and the
device key means losing the notes. External storage is never encrypted.

## Offline

A topic's materials are downloaded once and saved whole, as a single HTML file.
After that the network is not needed: the reader shows the text from the archive.
Nothing but an explicitly requested download ever goes online.

## The Obsidian plugin

It shows the program, the status of the open topic and a "time to review" mark in
the status bar, and opens a topic in the app on command. It never writes
anything. Point it at the app's data directory in the plugin settings. The status
shown is the recorded one — blocking and staleness are recomputed by the app, so
the precise answer is always there. With encryption on, the plugin sees no
statuses and says so.

## Where your data lives

- Programs stay wherever you put them; the app remembers the path in its
  registry.
- Settings, the program registry and the store (default notes, page archives,
  version history) live in `~/.config/tolearn` and `~/.local/share/tolearn`,
  identically on all three operating systems; exact paths and how to move them —
  [install/](install/README.md#where-your-data-lives).
- Uninstalling the app does not touch your data.

## Export

The "Export to Markdown" button on the program screen collects the whole program
into one file — table of contents, topics, notes. Answers and traps make it into
the file only for topics that were passed. The export never writes inside the
bundle itself.

# Notes

[Русский](../ru/notes.md) · **English**

## The problem

A note is one of the few things in this app that a person creates themselves.
Everything else follows from that: a note has to survive regenerating the bundle,
uninstalling the app and moving to another tool. Nobody writes notes that can be
lost along with the learning program.

## Where they live

**Not inside the bundle.** A bundle is regenerated wholesale (ADR-009), and
putting user data there means erasing it one day.

By default:

```
<app-data>/notes/<roadmap-id>/<topic-id>.md
```

If an external directory is set (an Obsidian vault, a Logseq folder, any other
directory), notes are written there and the app stops being their owner. The path
is configured per program: one program can write into a work vault, another into a
personal one.

## Format

Plain Markdown with YAML frontmatter:

```markdown
---
tolearn:
  roadmap: llm-agents-base
  topic: local-runtime
---

# Local runtime

Notes…
```

The link to a topic is held by the **frontmatter, not the path**. The file can be
renamed, moved to another folder of the vault, arranged into your own structure —
the binding does not break. The app indexes the directory by frontmatter.

There is nothing else technical in the file. Opened in any editor it looks like an
ordinary note, not like a database dump.

## Integrations

The integration here is the absence of one. The app writes `.md` into a folder;
anything that can read a folder of `.md` works without a single line of code:

| Tool | What is needed |
|---|---|
| Obsidian | point the notes directory at the vault |
| Logseq | the same; pages go into `pages/` |
| Zettlr, Typora, iA Writer, Zed, any editor | the same |
| git | the notes directory is an ordinary repository, versioning for free |

On top of that there is an "open in Obsidian" button via `obsidian://open?path=…`,
if Obsidian is installed. That is the only place where the app knows about a
specific tool, and it is optional.

### The Obsidian plugin

An optional add-on over the same folder (S54, the `obsidian/` package). It adds
nothing to the format and writes nothing: it reads `registry.yaml` and `progress.*`
from the data directory, finds `roadmap` and `topic` in the frontmatter of the open
note, and shows the program name, the topic status and a "time to review" mark in
the status bar when `next_review_at` has arrived. The "Open topic in tolearn"
command builds a `tolearn://topic?roadmap=…&topic=…` link.

The status shown is the **recorded** one: `blocked` is displayed as "not started",
because blocking and exam staleness are recomputed by the core across the whole
graph, and the plugin deliberately does not duplicate that logic. A discrepancy is
possible, and the real answer is always in the app.

The data directory and the language are set in the plugin's settings. If
`identity.age` sits next to the notes, the plugin does not silently show nothing —
it says the notes are encrypted and only the app can see the statuses.

A `tolearn://` link is parsed in the core and in the app and **only navigates**:
an unknown program, an unreachable bundle, a missing topic and any foreign value
are rejected with a code, changing nothing. Registering the scheme with the OS is
the installer's job (S57).

What there will **not** be: Notion, Evernote, Apple Notes, Google Keep. They all
require an API key, an account and a network request — which contradicts both "no
SDKs" and offline. People who use them sync the folder themselves.

## Two-way editing

A file can change from outside — that is the whole point. The app watches the
directory and picks up edits. If a file changed both outside and inside at the
same time, the app **neither merges nor picks**: it shows both versions and lets
you decide. Resolving that conflict silently means losing text, and the text was
written by a person.

Deleting a file from outside is not an error: the topic simply has no note again.

All of this describes notes in the clear — that is, the default mode. Turning
encryption on cancels two-way editing, see below.

## Encryption

Optional, off by default, specified by S53. Encryption and everything this
document is built on are direct opposites, so the boundaries are strict.

**Only the internal directory** `<app-data>/notes` is encrypted. External storage
(Obsidian, Logseq, a git repository) is never encrypted: the app does not own
someone else's directory and has no right to turn its contents into unreadable
files.

**The two modes are mutually exclusive.** A program either writes into an external
directory or encrypts the internal one. Enabling encryption while an external
directory is set is rejected with an explanation; setting an external directory
while encryption is on requires decrypting first.

What stops working while encryption is on is the price, and it is named before you
turn it on, not after:

| | Clear mode | Encryption |
|---|---|---|
| External editor | works | no |
| "Open in Obsidian" button | works | hidden |
| Picking up outside edits | works | no, the app owns the directory |
| Obsidian plugin (S54) | sees the files | sees nothing |

**Keys.** `age` (the Rust implementation `rage`, MIT/Apache-2.0). The store has
**two recipients**, and that is not redundancy but two different scenarios:

| Recipient | Where it lives | What for |
|---|---|---|
| device key | the OS keychain | everyday work: the app opens notes without asking anything |
| passphrase | nowhere, only in memory while typed | access when there is no device key: OS reinstall, keychain reset, a second computer |

The `age` format **forbids** putting a passphrase recipient and a key recipient
into the same file (`MixedRecipientAndPassphrase`), so the two entrances are made
by wrapping the key rather than by giving every file two recipients: notes are
encrypted with the device key, and the key itself is stored twice — in the OS
keychain and in `identity.age` next to the notes, under the passphrase. The
observable behaviour is the same: either entrance opens the store, losing both
loses the notes.

Hence the honest phrasing of what is lost. A forgotten passphrase with a working
OS keychain costs nothing — the notes open with the device key. Losing both means
losing the notes: there is no backdoor key, no recovery, and this is stated on the
enabling screen before, not after.

**File names and binding.** A file name in the encrypted directory is opaque
(`<random-id>.age`): `<topic-id>.md.age` would give away the contents of the
program to anyone who can see the directory. The binding is still held by the
frontmatter and not by the path — it is simply read after decryption. On access
the app walks the directory and builds a `roadmap/topic → file` mapping in memory;
the search index (S39) lives inside the same encrypted store, because an index in
the clear gives away exactly what the encryption was for.

Exporting a program summary writes **cleartext** Markdown into the chosen file:
that is what export is for. The app says so at the moment of export.

## What else gets written besides the topic note

- **A material mark** — a short line, "why read this / what turned out to be
  useful". Stored in the same topic file, in the `## Materials` section.
- **Post-exam analysis** — gaps from the verdict are appended to the note as a
  separate dated section, if the person pressed "add to note". Never
  automatically: the app does not write into someone else's file unasked.
- **Program summary** — an export of all the program's notes into one Markdown
  file, with a table of contents by stage. A separate action, into a chosen file.

## Orphaned notes

Regeneration can remove a topic or change its `id`. The note is **not deleted**.
It lands in the "notes without a topic" section, where it can be re-bound to an
existing topic or left as it is.

This is a deliberate bias: an extra file is a minor annoyance, a lost note is a
reason to stop using the app.

# Offline mode

[Русский](../ru/offline.md) · **English**

The "offline" switch downloads all of a program's materials to disk. After that
the network is not needed: links inside topics open from the local archive.

## What materials can be

A material's `type` field already says how to fetch it:

| `type` | Strategy |
|---|---|
| `docs` | `llms.txt` / the `.md` variant first, otherwise mirror the section |
| `article`, `post`, `guide` | monolith → one self-contained HTML + the reader |
| `spec`, `rfc` | monolith, usually clean HTML already |
| `repo` | `gix clone --depth 1` |
| `video` | external `yt-dlp`, if it is present in the system |
| `paper` | direct PDF download |

The `liveness` field is a ready-made filter. `paywall` and `login_required` are
not fetched at all: the result would be a login page, not the material. We show
them as "unavailable offline", honestly.

**What the reference bundle does not cover.** `examples/llm-agents-base` contains
only `type: docs` (32), `article` (4) and `repo` (1), and all 37 materials have
`liveness: ok`. So the `video`, `paper` and `spec` branches, along with skipping
on `paywall`/`login_required` — including the sample report at the end of this
document — are not reproduced by the only program we have. The value lists are
pinned down by S02 and the fixtures for those branches are created there; until
then they cannot be considered verified.

## monolith

[Y2Z/monolith](https://github.com/Y2Z/monolith) — Rust, CC0-1.0, used **as a
library**, not just as a CLI. It inlines CSS, images, fonts and scripts as data
URIs and returns a single HTML5 file.

Useful switches: `-I` isolate the document (forbids any outbound request via CSP),
`-j` drop JS, `-v` drop video, `-a` drop audio, `-b` base URL, `-d`/`-B` domain
allow- and denylist.

For learning materials `-I -j` is almost always right. JS is not needed in an
archived article, and isolation guarantees the saved page will not reach out to
the network.

**Limitation:** monolith has no JS engine. A page that renders itself on the
client is saved empty. The project's own docs suggest running such pages through
Chromium — that is, bundling Chromium.

## Prerendering with our own WebView

No need to bundle Chromium: it is already there. Tauri runs on the system WebView,
and we can use it as a renderer.

```
open a hidden window with the URL
  → wait for load (event + timeout)
  → run document.documentElement.outerHTML
  → hand the resulting HTML to monolith as a ready document
  → close the window
```

Zero new dependencies, zero extra megabytes, and monolith's limitation is gone.
The hidden window lives on a timeout and cannot hang the queue.

## The reader

`dom_smoothie` — a Rust port of readability.js, the same algorithm browsers use
for reader mode. It yields a title, an author and the main text. The reader is a
**second** layer on top of the archive, not a replacement: if extraction fails we
show the saved HTML as it is, not an empty screen.

Alternatives considered: `readable-readability` (simpler, weaker on complex
markup), `readability-js` (embeds QuickJS for the original Mozilla Readability —
an extra engine), `readability-rust`.

## Whole documentation sites

First we try what the site offers itself:

1. `/llms.txt` and `/llms-full.txt` — by now a de facto standard for developer
   documentation. Clean Markdown, the ideal offline format.
2. The `.md` variant of a page — many documentation generators serve one.
3. `sitemap.xml` — when a crawl is needed but the boundaries should be known
   upfront.

Only if none of that exists — the crawler. Hard limits: same domain, depth 2 by
default, a page-count ceiling, `robots.txt` respected. Links between saved pages
are rewritten to local ones.

## Video

We do not bundle `yt-dlp` — see
[ADR-004](../adr/004-video-external.md) (in Russian). We look for `yt-dlp` and
`ffmpeg` in `PATH`:

- absent — the button is disabled, with an explanation and the install command for
  the current OS next to it;
- present — we run it as an external process, show progress and allow cancelling.

`PATH` is parsed by the rules of its own OS: `;` as the separator on Windows and
`:` elsewhere, and the binary name carries `.exe` where the OS demands it.
Splitting on a colon under Windows would cut the path at the drive letter and find
nothing.

The default resolution cap is 720p: a 40-minute lecture at 1080p is hundreds of
megabytes, and slide legibility does not change.

## Cache

```
<app-data>/cache/objects/<sha256[0:2]>/<sha256>
<app-data>/cache/index.sqlite
```

The index: `url → sha256, type, fetched_at, size, etag, last_modified`.

Content addressing gives deduplication: the same RFC referenced by three programs
is stored once. The cache lives **outside** the bundle, because a bundle gets
regenerated and the cache has to survive that (ADR-005).

Revalidation uses conditional requests with `ETag` / `Last-Modified`. The disk
budget is configurable, eviction is LRU, and objects belonging to programs with
offline enabled are protected from eviction.

## The report

A download must end with an honest report, not with "done":

```
Downloaded  34
Skipped      6   paywall (4), login_required (2)
Failed       3   timeout (2), 404 (1)
```

Failed items are retried by a separate action, without re-downloading the
successful ones.

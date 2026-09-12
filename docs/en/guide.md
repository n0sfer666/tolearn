# User guide

[Русский](../ru/guide.md) · **English**

The short version of how to use the app. For installation see
[install/](install/README.md).

## Getting started

1. Take a ready learning program — a `.tolearn` file. You can build one from a
   program folder with `tolearn pack`, for example from
   [`examples/chiptune`](../../examples/chiptune); the CLI is built from source
   ([install/source.md](install/source.md)). Generating a program inside the app
   is removed for now and will come back one stage at a time.
2. Drop the file onto the home screen or pick it with the button. The app checks
   the package and names the reason if it refuses it.
3. Open the program, then a stage, and start.

## Screens

| Screen | What for |
|---|---|
| Program | the goal, the map, stages and subprograms |
| Stage | the stage text, practice and questions to read |
| Search | across the stages of every program in the library; a hit opens the stage at the right spot |
| Settings | language, LLM provider, disk budget |

## Where your data lives

- Settings live in `~/.config/tolearn` and data in `~/.local/share/tolearn`,
  identically on all three operating systems; exact paths and how to move them —
  [install/](install/README.md#where-your-data-lives).
- Uninstalling the app does not touch your data.
- The app keeps v2 programs in its own library, in
  `~/.local/share/tolearn/programs/`, and the search index over them next to
  it, in `search.yaml`: it rebuilds itself and is safe to delete. v1 program
  data — the registry `~/.config/tolearn/registry.yaml` and, under
  `~/.local/share/tolearn`, the
  `unpacked/`, `history/` and `offline/` directories and the `search-*` search
  index — is neither read nor removed by the app once it moves to v2: you can
  delete it by hand.

## Export

The "Export to Markdown" button on the program screen asks for a folder and puts
a folder named after the program into it: `index.md` with the contents by stages
and subprograms, one page per generated stage, and the pictures next to them.
Links are relative, so the folder opens in any Markdown editor or goes into git
as is. On a subprogram screen only that subprogram is exported. Reference
answers to the questions are never exported. The export never writes into a
non-empty folder or inside the app's library. The export is assembled in a
hidden folder next to the target, `.<name>.partial-<number>`, and renamed as a
whole at the end; if the app crashes mid-export, that hidden folder can be
removed by hand.

From a terminal, `tolearn export <program folder> <folder>` exports too. Unlike
the button, it writes straight into `<folder>` with no nested folder named after
the program, so `<folder>` must be empty or not exist yet. It never writes
inside any program folder.

## Generating from a terminal

`tolearn new "<request>" --level <level> --out <folder>` builds the program map
and its first stage into `<folder>/programs/<uuid>/`, not into the app's
library. The model provider comes from the app's settings and a remote provider's key
from the system keychain; another provider settings file is named with
`--provider <file>`. Without a network the command refuses with a reason and
never calls the model. `new` never writes into a folder that already holds a
program, nor into the app's own data folder.

While it runs, a line per step goes to standard error — time and tokens — and
at the end the output says where the program is, which stage was built and
which command to call next. A terminal has no window to draw diagrams in, so
diagrams stay Mermaid source, and the "схемы" step line says so.

`tolearn next <folder>` shows the fork variants after the last built stage,
`tolearn next <folder> --choice <number>` builds the chosen one. With `--json`
both commands print the summary as an object. `tolearn pack
<folder>/programs/<uuid> <file>` packs the finished program, and the app
imports it.

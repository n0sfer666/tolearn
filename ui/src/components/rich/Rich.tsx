import { For, Show } from "solid-js";
import { Dynamic } from "solid-js/web";

import Piece from "./Piece";
import Spans from "./Spans";
import type { Copier, Words } from "./Snip";
import type { Chunk } from "../../lib/rich";
import { rich, worded } from "../../lib/rich";

interface Props {
  text: string;
  words: Words;
  copy?: Copier;
}

function ordered(chunk: Chunk): boolean {
  return chunk.kind === "item" && chunk.ordered;
}

function listing(chunk: Chunk): boolean {
  return chunk.kind === "item";
}

function runs(chunks: Chunk[]): Chunk[][] {
  const made: Chunk[][] = [];
  for (const chunk of chunks) {
    const last = made.at(-1);
    const same = last !== undefined && listing(last[0]) && listing(chunk);
    if (same && ordered(last[0]) === ordered(chunk)) last.push(chunk);
    else made.push([chunk]);
  }
  return made;
}

export default function Rich(props: Props) {
  return (
    <For each={runs(rich(props.text))}>
      {(run) => (
        <Show
          when={listing(run[0])}
          fallback={<Piece chunk={run[0]} words={props.words} copy={props.copy} />}
        >
          <Dynamic component={ordered(run[0]) ? "ol" : "ul"} data-rich>
            <For each={run}>
              {(item) => (
                <li>
                  <Spans spans={worded(item)} words={props.words} copy={props.copy} />
                </li>
              )}
            </For>
          </Dynamic>
        </Show>
      )}
    </For>
  );
}

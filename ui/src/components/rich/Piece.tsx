import { Match, Switch } from "solid-js";
import { Dynamic } from "solid-js/web";

import Snip from "./Snip";
import Spans from "./Spans";
import type { Copier, Words } from "./Snip";
import type { Chunk } from "../../lib/rich";
import { worded } from "../../lib/rich";

type Head = Extract<Chunk, { kind: "head" }>;
type Fenced = Extract<Chunk, { kind: "code" }>;

interface Props {
  chunk: Chunk;
  words: Words;
  copy?: Copier;
}

const DEEPEST = 6;
const OFFSET = 2;

function heading(chunk: Chunk): Head | undefined {
  return chunk.kind === "head" ? chunk : undefined;
}

function fenced(chunk: Chunk): Fenced | undefined {
  return chunk.kind === "code" ? chunk : undefined;
}

export default function Piece(props: Props) {
  const said = () => (
    <Spans spans={worded(props.chunk)} words={props.words} copy={props.copy} />
  );

  return (
    <Switch fallback={<p>{said()}</p>}>
      <Match when={heading(props.chunk)}>
        {(head) => (
          <Dynamic component={`h${Math.min(head().level + OFFSET, DEEPEST)}`}>{said()}</Dynamic>
        )}
      </Match>
      <Match when={props.chunk.kind === "quote"}>
        <blockquote>{said()}</blockquote>
      </Match>
      <Match when={fenced(props.chunk)}>
        {(block) => (
          <pre data-rich-code>
            <Snip text={block().text} words={props.words} copy={props.copy} />
          </pre>
        )}
      </Match>
    </Switch>
  );
}

import { For, Match, Show, Switch } from "solid-js";
import { Dynamic } from "solid-js/web";

import type { Block } from "../../ipc";

interface Props {
  blocks: readonly Block[];
}

interface OneProps {
  block: Block;
}

const DEEPEST = 6;
const OFFSET = 2;

function runs(blocks: readonly Block[]): Block[][] {
  const made: Block[][] = [];
  for (const block of blocks) {
    const last = made.at(-1);
    if (last !== undefined && last[0].kind === "item" && block.kind === "item") last.push(block);
    else made.push([block]);
  }
  return made;
}

function One(props: OneProps) {
  const tag = () => `h${Math.min(props.block.level + OFFSET, DEEPEST)}`;

  return (
    <Switch fallback={<p>{props.block.text}</p>}>
      <Match when={props.block.kind === "heading"}>
        <Dynamic component={tag()}>{props.block.text}</Dynamic>
      </Match>
      <Match when={props.block.kind === "quote"}>
        <blockquote>{props.block.text}</blockquote>
      </Match>
      <Match when={props.block.kind === "code"}>
        <pre data-code>
          <code>{props.block.text}</code>
        </pre>
      </Match>
    </Switch>
  );
}

export default function Blocks(props: Props) {
  return (
    <For each={runs(props.blocks)}>
      {(run) => (
        <Show when={run[0].kind === "item"} fallback={<One block={run[0]} />}>
          <ul>
            <For each={run}>{(item) => <li>{item.text}</li>}</For>
          </ul>
        </Show>
      )}
    </For>
  );
}

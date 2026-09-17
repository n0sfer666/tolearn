import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import { clocked } from "../../lib/clocked";
import type { Generation } from "../../lib/generation";
import type { Mark } from "../../lib/marks";

interface Props {
  text: Dictionary;
  seen: Mark[];
  work: Generation;
}

export default function Steps(props: Props) {
  const current = () => (props.work.running() ? props.seen.find((mark) => !mark.done)?.key : undefined);
  const timed = (mark: Mark) => (mark.spent === null ? undefined : clocked(mark.spent, props.text));

  return (
    <Show when={props.seen.length > 0}>
      <ol data-steps>
        <For each={props.seen}>
          {(mark) => {
            const busy = () => mark.key === current();
            return (
              <li data-mark={mark.key} data-done={mark.done ? true : undefined} data-busy={busy() ? true : undefined}>
                <Show when={busy()}>
                  <span data-spin aria-hidden="true" />
                </Show>
                <span data-label>{busy() ? props.work.said() : mark.label}</span>
                <Show when={busy()}>
                  <span data-elapsed>{clocked(props.work.spent(), props.text)}</span>
                </Show>
                <Show when={!busy() && timed(mark)}>{(at) => <span data-at>{at()}</span>}</Show>
              </li>
            );
          }}
        </For>
      </ol>
    </Show>
  );
}

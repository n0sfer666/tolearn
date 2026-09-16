import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { Mark } from "../../lib/marks";

interface Props {
  text: Dictionary;
  seen: Mark[];
}

export default function Steps(props: Props) {
  const spent = (mark: Mark) => {
    if (mark.spent === null) return undefined;
    return props.text.generate.seconds.replace("{n}", String(Math.round(mark.spent / 1000)));
  };

  return (
    <Show when={props.seen.length > 0}>
      <ol data-steps>
        <For each={props.seen}>
          {(mark) => (
            <li data-mark={mark.key} data-done={mark.done ? true : undefined}>
              <span>{mark.label}</span>
              <Show when={spent(mark)}>{(text) => <span data-at>{text()}</span>}</Show>
            </li>
          )}
        </For>
      </ol>
    </Show>
  );
}

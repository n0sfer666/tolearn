import { For } from "solid-js";

interface Props {
  lines: readonly string[];
}

export default function HintLines(props: Props) {
  return (
    <span data-hint-rules role="list">
      <For each={props.lines}>
        {(line) => (
          <span data-hint-rule role="listitem">
            {line}
          </span>
        )}
      </For>
    </span>
  );
}

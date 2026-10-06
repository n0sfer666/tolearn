import { For } from "solid-js";

interface Props {
  keys: string[];
  label: (key: string) => string;
  current: string;
  pick: (key: string) => void;
}

export default function Tabs(props: Props) {
  return (
    <div data-tabs>
      <For each={props.keys}>
        {(key) => (
          <button
            type="button"
            data-tab={key}
            aria-pressed={props.current === key}
            onClick={() => props.pick(key)}
          >
            {props.label(key)}
          </button>
        )}
      </For>
    </div>
  );
}

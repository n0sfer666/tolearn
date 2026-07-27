import { For, createSignal } from "solid-js";

interface Props {
  items: string[];
}

export default function Acceptance(props: Props) {
  const [checked, setChecked] = createSignal<number[]>([]);
  const has = (at: number) => checked().includes(at);
  const toggle = (at: number) =>
    setChecked((was) => (was.includes(at) ? was.filter((one) => one !== at) : [...was, at]));

  return (
    <ul>
      <For each={props.items}>
        {(item, at) => (
          <li>
            <label>
              <input type="checkbox" checked={has(at())} onChange={() => toggle(at())} />
              {item}
            </label>
          </li>
        )}
      </For>
    </ul>
  );
}

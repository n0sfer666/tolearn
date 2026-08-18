import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { SweepSettledView } from "../../ipc";
import { label } from "../status";

interface Props {
  text: Dictionary;
  settled: SweepSettledView[];
}

export default function Settled(props: Props) {
  return (
    <section data-settled>
      <h3>{props.text.sweep.accepted}</h3>
      <ul>
        <For each={props.settled}>
          {(item) => (
            <li data-settled-topic={item.topic}>
              <span>{item.title}</span>
              <span data-result>{item.result}</span>
              <span data-status>{label(props.text, item.status)}</span>
            </li>
          )}
        </For>
      </ul>
    </section>
  );
}

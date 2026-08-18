import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { SweepLegView } from "../../ipc";

interface Props {
  text: Dictionary["sweep"];
  legs: SweepLegView[];
}

export default function Legs(props: Props) {
  return (
    <section data-legs>
      <h3>{props.text.legs}</h3>
      <ul>
        <For each={props.legs}>
          {(leg) => (
            <li data-leg={leg.topic}>
              <span>{leg.title}</span>
              <span data-asked>
                {leg.asked} / {leg.total}
              </span>
              <span data-verdict>
                {leg.verdict === null ? props.text.waiting : props.text.verdict}
              </span>
            </li>
          )}
        </For>
      </ul>
    </section>
  );
}

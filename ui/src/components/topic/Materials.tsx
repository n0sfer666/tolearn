import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { MaterialView } from "../../ipc";

interface Props {
  text: Dictionary;
  materials: MaterialView[];
}

export default function Materials(props: Props) {
  return (
    <ul>
      <For each={props.materials}>
        {(material) => (
          <li data-material={material.kind}>
            <a href={material.url}>{material.title}</a>
            <span data-tier>{material.tier}</span>
            <span data-kind>{material.kind}</span>
            <span data-offline>{props.text.topic.offline}</span>
            <Show when={material.stale}>
              <span data-stale>{props.text.topic.stale}</span>
            </Show>
            <Show when={material.delta}>{(delta) => <span data-delta>{delta()}</span>}</Show>
          </li>
        )}
      </For>
    </ul>
  );
}

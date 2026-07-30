import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { MaterialView } from "../../ipc";

interface Props {
  text: Dictionary;
  materials: MaterialView[];
}

export default function Materials(props: Props) {
  const [stale, setStale] = createSignal(false);
  const shown = () =>
    stale() ? props.materials : props.materials.filter((material) => !material.stale);

  return (
    <>
      <div class="head">
        <h2>{props.text.topic.materials}</h2>
        <label data-toggle>
          <input
            type="checkbox"
            data-show-stale
            checked={stale()}
            onChange={(event) => setStale(event.currentTarget.checked)}
          />
          {props.text.topic.showStale}
        </label>
      </div>
      <ul>
        <For each={shown()}>
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
    </>
  );
}

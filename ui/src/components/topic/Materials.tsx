import { For, Show, createSignal } from "solid-js";

import Unload from "./Unload";
import type { Dictionary } from "../../i18n/ru";
import type { Locale } from "../../i18n";
import type { MaterialView, UnloadView } from "../../ipc";
import type { Transport } from "../../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  bundle: string;
  topic: string;
  materials: MaterialView[];
  unload: UnloadView;
  call?: Transport;
  onSaved?: () => void;
}

export default function Materials(props: Props) {
  const [stale, setStale] = createSignal(false);
  const shown = () =>
    stale() ? props.materials : props.materials.filter((material) => !material.stale);
  const reader = (url: string) =>
    `/${props.locale}/read/?url=${encodeURIComponent(url)}&program=${encodeURIComponent(props.bundle)}&topic=${encodeURIComponent(props.topic)}`;

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
        <Unload
          text={props.text}
          bundle={props.bundle}
          topic={props.topic}
          unload={props.unload}
          materials={props.materials}
          call={props.call}
          onDone={props.onSaved}
        />
      </div>
      <ol data-materials>
        <For each={shown()}>
          {(material) => (
            <li data-material={material.kind}>
              <a href={material.url}>{material.title}</a>
              <span data-tier>{material.tier}</span>
              <span data-kind>{material.kind}</span>
              <Show
                when={material.offline === "saved"}
                fallback={<span data-offline>{props.text.topic.offline}</span>}
              >
                <a data-offline-open href={reader(material.url)}>
                  {props.text.offline.open}
                </a>
              </Show>
              <Show when={material.stale}>
                <span data-stale>{props.text.topic.stale}</span>
              </Show>
              <Show when={material.delta}>{(delta) => <span data-delta>{delta()}</span>}</Show>
              <Show when={material.note}>{(note) => <p data-note>{note()}</p>}</Show>
            </li>
          )}
        </For>
      </ol>
    </>
  );
}

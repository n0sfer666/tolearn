import { For, Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { MaterialView, UnloadView } from "../../ipc";
import { megabytes } from "../../lib/size";
import { moment } from "../../lib/moment";
import { quiet } from "../../lib/ipc";
import { unloading } from "../../lib/unload";
import type { Transport } from "../../lib/ipc";

interface Props {
  text: Dictionary;
  bundle: string;
  topic: string;
  unload: UnloadView;
  materials: MaterialView[];
  call?: Transport;
  onDone?: () => void;
}

export default function Unload(props: Props) {
  const call = () => props.call ?? quiet;
  const run = unloading(call, props.text.toast.broke, () => props.onDone?.());

  const start = () => run.start({ bundle: props.bundle, topic: props.topic });
  const stamp = () => (props.unload.state === "fresh" ? props.unload.checked_at : null);
  const locked = () => !run.broken() && stamp() !== null;
  const kept = () => props.unload.state === "stale" || props.unload.state === "unchecked";
  const named = (url: string) =>
    props.materials.find((material) => material.url === url)?.title ?? url;

  const label = () => {
    if (run.broken()) return props.text.offline.again;
    const at = stamp();
    if (at !== null) return `${props.text.offline.actual} ${moment(at)}`;
    return kept() ? props.text.offline.update : props.text.offline.save;
  };

  return (
    <div data-unload>
      <Show
        when={run.running()}
        fallback={
          <button
            type="button"
            data-save-offline
            data-unload-state={props.unload.state}
            disabled={locked()}
            onClick={start}
          >
            {label()}
          </button>
        }
      >
        <span data-unload-progress>
          {props.text.offline.saving} {run.state()?.done ?? 0}/{run.state()?.total ?? 0}
        </span>
        <button type="button" data-unload-stop onClick={run.stop}>
          {props.text.offline.stop}
        </button>
      </Show>
      <Show when={run.done()}>
        {(out) => (
          <>
            <span data-unload-report>
              <Show when={out().cancelled}>{props.text.offline.cancelled}, </Show>
              {out().saved.length} {props.text.offline.saved}, {out().skipped.length}{" "}
              {props.text.offline.skipped}, {out().failed.length} {props.text.offline.failed}
              <Show when={out().bytes > 0}>
                {", "}
                {megabytes(out().bytes)} {props.text.offline.bytes}
              </Show>
            </span>
            <Show when={out().saved.length > 0}>
              <ul data-unload-saved>
                <For each={out().saved}>{(url) => <li>{named(url)}</li>}</For>
              </ul>
            </Show>
            <Show when={out().failed.length > 0}>
              <ul data-unload-failed>
                <For each={out().failed}>
                  {(left) => (
                    <li>
                      {named(left.url)} — {left.why}
                    </li>
                  )}
                </For>
              </ul>
            </Show>
          </>
        )}
      </Show>
    </div>
  );
}

import { For, Show, createSignal, onCleanup } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { MaterialView, OfflineStateOut, UnloadView } from "../../ipc";
import { explain, toast } from "../../lib/toast";
import { moment } from "../../lib/moment";
import { quiet } from "../../lib/ipc";
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

const STEP = 300;
const MEGABYTE = 1024 * 1024;

const megabytes = (bytes: number) => Math.max(1, Math.round(bytes / MEGABYTE));

export default function Unload(props: Props) {
  const call = () => props.call ?? quiet;
  const [job, setJob] = createSignal("");
  const [state, setState] = createSignal<OfflineStateOut | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const halt = () => {
    clearInterval(timer);
    timer = undefined;
  };
  onCleanup(halt);

  const broke = (error: unknown) => {
    halt();
    toast("error", explain(error) || props.text.toast.broke);
  };

  const look = () => {
    void (async () => {
      try {
        const out = await call()("offline_state", { job: job() });
        setState(out);
        if (!out.finished) return;
        halt();
        props.onDone?.();
      } catch (error) {
        setJob("");
        broke(error);
      }
    })();
  };

  const start = () => {
    const again = broken() ? job() : null;
    void (async () => {
      try {
        const out = await call()("save_offline", {
          bundle: props.bundle,
          topic: props.topic,
          again,
        });
        setJob(out.job);
        setState(null);
        halt();
        timer = setInterval(look, STEP);
        look();
      } catch (error) {
        broke(error);
      }
    })();
  };

  const stop = () => {
    void (async () => {
      try {
        await call()("stop_offline", { job: job() });
      } catch (error) {
        broke(error);
      }
    })();
  };

  const done = () => {
    const out = state();
    return out !== null && out.finished ? out : null;
  };
  const running = () => job() !== "" && done() === null;
  const broken = () => (done()?.failed.length ?? 0) > 0;
  const stamp = () => (props.unload.state === "fresh" ? props.unload.checked_at : null);
  const locked = () => !broken() && stamp() !== null;
  const kept = () => props.unload.state === "stale" || props.unload.state === "unchecked";
  const named = (url: string) =>
    props.materials.find((material) => material.url === url)?.title ?? url;

  const label = () => {
    if (broken()) return props.text.offline.again;
    const at = stamp();
    if (at !== null) return `${props.text.offline.actual} ${moment(at)}`;
    return kept() ? props.text.offline.update : props.text.offline.save;
  };

  return (
    <div data-unload>
      <Show
        when={running()}
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
          {props.text.offline.saving} {state()?.done ?? 0}/{state()?.total ?? 0}
        </span>
        <button type="button" data-unload-stop onClick={stop}>
          {props.text.offline.stop}
        </button>
      </Show>
      <Show when={done()}>
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

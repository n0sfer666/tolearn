import { Show, createSignal, onCleanup } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { OfflineStateOut } from "../../ipc";
import { transport } from "../../lib/ipc";
import type { Transport } from "../../lib/ipc";

interface Props {
  text: Dictionary;
  bundle: string;
  topic: string;
  call?: Transport;
  onDone?: () => void;
}

const STEP = 300;
const MEGABYTE = 1024 * 1024;

const megabytes = (bytes: number) => Math.max(1, Math.round(bytes / MEGABYTE));

export default function Unload(props: Props) {
  const call = () => props.call ?? transport;
  const [job, setJob] = createSignal("");
  const [state, setState] = createSignal<OfflineStateOut | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const halt = () => {
    clearInterval(timer);
    timer = undefined;
  };
  onCleanup(halt);

  const look = () => {
    void (async () => {
      const out = await call()("offline_state", { job: job() });
      setState(out);
      if (!out.finished) return;
      halt();
      props.onDone?.();
    })();
  };

  const start = () => {
    const again = broken() ? job() : null;
    void (async () => {
      const out = await call()("save_offline", { bundle: props.bundle, topic: props.topic, again });
      setJob(out.job);
      setState(null);
      halt();
      timer = setInterval(look, STEP);
      look();
    })();
  };

  const stop = () => {
    void call()("stop_offline", { job: job() });
  };

  const done = () => {
    const out = state();
    return out !== null && out.finished ? out : null;
  };
  const running = () => job() !== "" && done() === null;
  const broken = () => (done()?.failed.length ?? 0) > 0;

  return (
    <div data-unload>
      <Show
        when={running()}
        fallback={
          <button type="button" data-save-offline onClick={start}>
            {broken() ? props.text.offline.again : props.text.offline.save}
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
          <span data-unload-report>
            <Show when={out().cancelled}>{props.text.offline.cancelled}, </Show>
            {out().saved.length} {props.text.offline.saved}, {out().skipped.length}{" "}
            {props.text.offline.skipped}, {out().failed.length} {props.text.offline.failed}
            <Show when={out().bytes > 0}>
              {", "}
              {megabytes(out().bytes)} {props.text.offline.bytes}
            </Show>
          </span>
        )}
      </Show>
    </div>
  );
}

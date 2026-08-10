import { For, Show, createSignal } from "solid-js";

import Price from "./Price";
import type { Dictionary } from "../../i18n/ru";
import type { OfflineCostOut, OfflineStateOut } from "../../ipc";
import { explain, toast } from "../../lib/toast";
import { megabytes } from "../../lib/size";
import { quiet } from "../../lib/ipc";
import { unloading } from "../../lib/unload";
import type { Transport } from "../../lib/ipc";

interface Props {
  text: Dictionary;
  bundle: string;
  call?: Transport;
}

const EMPTY: OfflineStateOut = {
  total: 0,
  done: 0,
  current: "",
  title: "",
  topic: "",
  topic_at: 0,
  topics: 0,
  finished: false,
  cancelled: false,
  bytes: 0,
  saved: [],
  skipped: [],
  failed: [],
};

export default function Unload(props: Props) {
  const call = () => props.call ?? quiet;
  const [price, setPrice] = createSignal<OfflineCostOut | null>(null);
  const [counting, setCounting] = createSignal(false);
  const run = unloading(call, props.text.toast.broke);

  const go = () => {
    setPrice(null);
    setCounting(false);
    run.start({ bundle: props.bundle, topic: null });
  };

  const back = () => {
    setPrice(null);
    setCounting(false);
  };

  const ask = () => {
    setCounting(true);
    void (async () => {
      try {
        const cost = await call()("offline_cost", { bundle: props.bundle });
        if (!cost.tight) return go();
        setCounting(false);
        setPrice(cost);
      } catch (error) {
        setCounting(false);
        toast("error", explain(error) || props.text.toast.broke);
      }
    })();
  };

  const label = () => {
    if (counting()) return props.text.offline.counting;
    return run.broken() ? props.text.offline.again : props.text.offline.whole;
  };

  const where = (out: OfflineStateOut) => {
    const topic = `${props.text.offline.topicAt} ${out.topic_at} ${props.text.offline.of} ${out.topics}`;
    const at = Math.min(out.done + 1, Math.max(out.total, 1));
    const piece = `${props.text.offline.piece} ${at} ${props.text.offline.of} ${out.total}`;
    const named = out.title === "" ? out.current : out.title;
    return [topic, out.topic, piece, named].filter((part) => part !== "").join(" · ");
  };

  return (
    <div data-whole-unload>
      <Show
        when={run.running()}
        fallback={
          <button
            type="button"
            data-unload-whole
            disabled={counting() || price() !== null}
            onClick={run.broken() ? go : ask}
          >
            {label()}
          </button>
        }
      >
        <span data-whole-progress>
          {props.text.offline.saving}: {where(run.state() ?? EMPTY)}
        </span>
        <button type="button" data-whole-stop onClick={run.stop}>
          {props.text.offline.stop}
        </button>
      </Show>
      <Show when={price()}>
        {(cost) => <Price text={props.text} cost={cost()} onGo={go} onBack={back} />}
      </Show>
      <Show when={run.done()}>
        {(out) => (
          <>
            <span data-whole-report>
              <Show when={out().cancelled}>{props.text.offline.cancelled}, </Show>
              {out().saved.length} {props.text.offline.saved}, {out().skipped.length}{" "}
              {props.text.offline.skipped}, {out().failed.length} {props.text.offline.failed}
              <Show when={out().bytes > 0}>
                {", "}
                {megabytes(out().bytes)} {props.text.offline.bytes}
              </Show>
            </span>
            <Show when={out().failed.length > 0}>
              <ul data-whole-failed>
                <For each={out().failed}>
                  {(left) => (
                    <li>
                      {left.title} — {left.why}
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

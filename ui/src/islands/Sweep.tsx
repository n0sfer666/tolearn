import { Show, createSignal, onMount } from "solid-js";

import Legs from "../components/sweep/Legs";
import Pool from "../components/sweep/Pool";
import Reply from "../components/exam/Reply";
import Settled from "../components/sweep/Settled";
import Talk from "../components/exam/Talk";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { SweepSettledView, SweepStateOut } from "../ipc";
import { explain, toast } from "../lib/toast";
import { query } from "../lib/query";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  today?: string;
  call?: Transport;
}

const DEFAULT_TOPICS = 5;

export default function Sweep(props: Props) {
  const call = () => props.call ?? quiet;
  const program = () => props.program ?? query("program");
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const words = () => props.text.sweep;

  const [view, setView] = createSignal<SweepStateOut | null>(null);
  const [enabled, setEnabled] = createSignal<boolean | null>(null);
  const [draft, setDraft] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [topics, setTopics] = createSignal(DEFAULT_TOPICS);
  const [settled, setSettled] = createSignal<SweepSettledView[] | null>(null);

  const broke = (error: unknown) => toast("error", explain(error) || props.text.toast.broke);
  const where = () => ({ bundle: program(), today: today() });

  const step = (run: () => Promise<SweepStateOut>) => {
    void (async () => {
      setBusy(true);
      try {
        setView(await run());
        setDraft("");
      } catch (error) {
        broke(error);
      } finally {
        setBusy(false);
      }
    })();
  };

  onMount(() => {
    void (async () => {
      try {
        const out = await call()("provider", {
          save: null,
          key: null,
          forget: false,
          check: false,
          probe: false,
        });
        setEnabled(out.provider.enabled);
      } catch (error) {
        setEnabled(false);
        broke(error);
      }
    })();
    step(() => call()("sweep_state", where()));
  });

  const start = (restart: boolean) => {
    setSettled(null);
    step(() => call()("sweep_start", { ...where(), topics: topics(), restart }));
  };

  const accept = () => {
    void (async () => {
      setBusy(true);
      try {
        const out = await call()("sweep_accept", where());
        setSettled(out.settled);
        setView(await call()("sweep_state", where()));
      } catch (error) {
        broke(error);
      } finally {
        setBusy(false);
      }
    })();
  };

  return (
    <article data-sweep>
      <Show when={enabled() === false}>
        <p data-sweep-off>{words().off}</p>
      </Show>

      <Show when={view()} keyed>
        {(live) => (
          <Show
            when={live.open}
            fallback={
              <Pool
                text={words()}
                ready={live.ready}
                topics={topics()}
                busy={busy() || enabled() !== true}
                onTopics={setTopics}
                onStart={() => start(false)}
              />
            }
          >
            <Show when={live.stale}>
              <p data-stale>{words().stale}</p>
            </Show>
            <Legs text={words()} legs={live.legs} />
            <Talk text={props.text.dialog} log={live.log} />
            <Show when={!live.done}>
              <Reply
                text={props.text.dialog}
                view={live}
                draft={draft()}
                busy={busy()}
                onDraft={setDraft}
                onSay={() => step(() => call()("sweep_say", { ...where(), text: draft() }))}
                onHint={() => step(() => call()("sweep_hint", where()))}
                onFinish={() => step(() => call()("sweep_finish", where()))}
                onRestart={() => start(true)}
              />
            </Show>
            <Show when={live.done}>
              <p class="row">
                <button type="button" data-accept disabled={busy()} onClick={accept}>
                  {busy() ? words().accepting : words().accept}
                </button>
              </p>
            </Show>
          </Show>
        )}
      </Show>

      <Show when={settled()}>
        {(list) => <Settled text={props.text} settled={list()} />}
      </Show>
    </article>
  );
}

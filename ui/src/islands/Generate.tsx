import { Show, createSignal, onCleanup, onMount } from "solid-js";

import Ask from "../components/generate/Ask";
import Going from "../components/generate/Going";
import Ready from "../components/generate/Ready";
import Resume from "../components/generate/Resume";
import Tile from "../components/generate/Tile";
import type { Dictionary } from "../i18n/ru";
import type { GenerateIn, GenerateStateOut } from "../ipc";
import { accepting } from "../lib/accepting";
import { drafting } from "../lib/drafting";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { explain, toast } from "../lib/toast";

interface Props {
  text: Dictionary["generate"];
  locale: string;
  today?: string;
  call?: Transport;
  open?: (href: string) => void;
  step?: number;
}

const STEP = 300;

export default function Generate(props: Props) {
  const call = () => props.call ?? quiet;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);

  const [enabled, setEnabled] = createSignal<boolean | null>(null);
  const [asking, setAsking] = createSignal(false);
  const [job, setJob] = createSignal("");
  const [live, setLive] = createSignal<GenerateStateOut | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const halt = () => {
    clearInterval(timer);
    timer = undefined;
  };
  onCleanup(halt);

  const broke = (error: unknown) => {
    halt();
    toast("error", explain(error) || props.text.broke);
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
        peek();
      } catch (error) {
        setEnabled(false);
        broke(error);
      }
    })();
  });

  const watch = (name: string) => {
    setAsking(false);
    clear();
    setLive(null);
    hide();
    setJob(name);
    halt();
    timer = setInterval(look, props.step ?? STEP);
    look();
  };

  const { draft, look: peek, take, forget: unpin, clear: hide } = drafting({ call, watch, broke });

  const look = () => {
    void (async () => {
      try {
        const out = await call()("generate_state", { job: job() });
        setLive(out);
        if (out.finished) halt();
      } catch (error) {
        setJob("");
        broke(error);
      }
    })();
  };

  const start = (request: GenerateIn) => {
    void (async () => {
      try {
        const out = await call()("generate", request);
        watch(out.job);
      } catch (error) {
        broke(error);
      }
    })();
  };

  const tell = (name: "generate_go" | "generate_stop") => {
    void (async () => {
      try {
        await call()(name, { job: job() });
      } catch (error) {
        broke(error);
      }
    })();
  };

  const close = () => {
    halt();
    setJob("");
    setLive(null);
    clear();
  };

  const drop = () => {
    tell("generate_stop");
    close();
  };

  const { busy, refused, accept, clear } = accepting({
    call,
    locale: () => props.locale,
    today,
    job,
    open: (href) => (props.open ?? ((where: string) => window.location.assign(where)))(href),
    close,
    broke,
  });

  const summary = () => live()?.summary ?? null;
  const shown = () => asking() || job() !== "";

  return (
    <div data-generate>
      <Show when={!shown()}>
        <Tile text={props.text} enabled={enabled()} onOpen={() => setAsking(true)} />
        <Show when={draft()}>
          {(kept) => (
            <Resume text={props.text} draft={kept()} onTake={take} onDrop={unpin} />
          )}
        </Show>
      </Show>

      <Show when={shown()}>
        <div role="dialog" aria-modal="true" aria-label={props.text.title}>
          <h3>{props.text.title}</h3>

          <Show when={asking()}>
            <p>{props.text.lead}</p>
            <Ask text={props.text} onStart={start} onClose={() => setAsking(false)} />
          </Show>

          <Show when={job() !== ""}>
            <Show
              when={summary()}
              fallback={
                <Going
                  text={props.text}
                  live={live()}
                  onGo={() => tell("generate_go")}
                  onStop={() => tell("generate_stop")}
                  onClose={drop}
                />
              }
            >
              {(view) => (
                <Ready
                  text={props.text}
                  summary={view()}
                  seconds={live()?.seconds ?? 0}
                  busy={busy()}
                  refused={refused()}
                  onAccept={accept}
                  onClose={drop}
                />
              )}
            </Show>
          </Show>
        </div>
      </Show>
    </div>
  );
}

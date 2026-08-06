import { Show, createSignal, onCleanup, onMount } from "solid-js";

import Ask from "../components/generate/Ask";
import Going from "../components/generate/Going";
import Ready from "../components/generate/Ready";
import Tile from "../components/generate/Tile";
import type { Dictionary } from "../i18n/ru";
import type { GenerateIn, GenerateStateOut } from "../ipc";
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
  const [busy, setBusy] = createSignal(false);
  const [refused, setRefused] = createSignal<string[]>([]);
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
      } catch (error) {
        setEnabled(false);
        broke(error);
      }
    })();
  });

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
        setAsking(false);
        setRefused([]);
        setLive(null);
        setJob(out.job);
        halt();
        timer = setInterval(look, props.step ?? STEP);
        look();
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
    setRefused([]);
  };

  const drop = () => {
    tell("generate_stop");
    close();
  };

  const opened = async (id: string | null) => {
    const { programs } = await call()("programs", { today: today() });
    const card = programs.find((program) => program.id === id);
    if (card === undefined) return `/${props.locale}/`;
    return `/${props.locale}/program/?program=${encodeURIComponent(card.path)}`;
  };

  const accept = () => {
    void (async () => {
      setBusy(true);
      setRefused([]);
      try {
        const done = await call()("generate_accept", { job: job(), today: today() });
        if (!done.ok) {
          setRefused(done.violations.map((violation) => violation.message));
          return;
        }
        const href = await opened(done.id);
        close();
        (props.open ?? ((where: string) => window.location.assign(where)))(href);
      } catch (error) {
        broke(error);
      } finally {
        setBusy(false);
      }
    })();
  };

  const summary = () => live()?.summary ?? null;
  const shown = () => asking() || job() !== "";

  return (
    <div data-generate>
      <Show when={!shown()}>
        <Tile text={props.text} enabled={enabled()} onOpen={() => setAsking(true)} />
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

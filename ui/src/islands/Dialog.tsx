import { Show, createSignal, onMount } from "solid-js";

import Parsed from "../components/exam/Parsed";
import Reply from "../components/exam/Reply";
import Talk from "../components/exam/Talk";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { ApplyVerdictOut, ExamStateOut, VerdictView } from "../ipc";
import { explain, toast } from "../lib/toast";
import { label } from "../components/status";
import { query } from "../lib/query";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  topic?: string;
  today?: string;
  call?: Transport;
}

export default function Dialog(props: Props) {
  const call = () => props.call ?? quiet;
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const words = () => props.text.dialog;

  const [view, setView] = createSignal<ExamStateOut | null>(null);
  const [enabled, setEnabled] = createSignal<boolean | null>(null);
  const [draft, setDraft] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [parsed, setParsed] = createSignal<VerdictView | null>(null);
  const [applied, setApplied] = createSignal<ApplyVerdictOut | null>(null);

  const broke = (error: unknown) => toast("error", explain(error) || props.text.toast.broke);
  const where = () => ({ bundle: program(), topic: id() });

  const took = (out: ExamStateOut) => {
    setView(out);
    setDraft("");
    if (out.verdict !== null) void read(out.verdict);
  };

  const read = async (text: string) => {
    try {
      setParsed(await call()("parse_verdict", { ...where(), text }));
    } catch {
      setParsed(null);
      toast("error", props.text.exam.broken);
    }
  };

  const step = (run: () => Promise<ExamStateOut>) => {
    void (async () => {
      setBusy(true);
      try {
        took(await run());
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
    step(() => call()("exam_state", where()));
  });

  const start = (restart: boolean) => {
    setParsed(null);
    setApplied(null);
    step(() => call()("exam_start", { ...where(), restart }));
  };

  const apply = () => {
    const text = view()?.verdict;
    if (text === undefined || text === null) return;
    void (async () => {
      try {
        setApplied(await call()("apply_verdict", { ...where(), text, today: today() }));
      } catch (error) {
        broke(error);
      }
    })();
  };

  const paper = () =>
    `/${props.locale}/exam/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;
  const review = () =>
    `/${props.locale}/review/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  return (
    <article data-dialog>
      <Show when={enabled() === false}>
        <p data-dialog-off>
          {words().off} <a href={paper()}>{words().paper}</a>
        </p>
      </Show>

      <Show when={view()} keyed>
        {(live) => (
          <Show
            when={live.open}
            fallback={
              <p>
                <button
                  type="button"
                  data-start
                  disabled={busy() || enabled() !== true}
                  onClick={() => start(false)}
                >
                  {words().start}
                </button>
              </p>
            }
          >
            <Show when={live.stale}>
              <p data-stale>{words().stale}</p>
            </Show>
            <Talk text={words()} log={live.log} />
            <Show when={live.verdict === null}>
              <Reply
                text={words()}
                view={live}
                draft={draft()}
                busy={busy()}
                onDraft={setDraft}
                onSay={() => step(() => call()("exam_say", { ...where(), text: draft() }))}
                onHint={() => step(() => call()("exam_hint", where()))}
                onFinish={() => step(() => call()("exam_finish", { ...where(), today: today() }))}
                onRestart={() => start(true)}
              />
            </Show>
          </Show>
        )}
      </Show>

      <Show when={parsed()}>
        {(verdict) => (
          <>
            <Parsed text={props.text} verdict={verdict()} />
            <p>
              <button type="button" data-apply onClick={apply}>
                {props.text.exam.apply}
              </button>
            </p>
          </>
        )}
      </Show>

      <Show when={applied()}>
        {(out) => (
          <section data-applied>
            <p>
              {props.text.exam.applied}: {label(props.text, out().status)}
            </p>
            <a href={review()} data-review-link>
              {props.text.exam.review}
            </a>
          </section>
        )}
      </Show>
    </article>
  );
}

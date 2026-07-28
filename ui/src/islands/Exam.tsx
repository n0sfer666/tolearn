import { Show, createSignal, onMount } from "solid-js";

import Parsed from "../components/exam/Parsed";
import type { ApplyVerdictOut, VerdictView } from "../ipc";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import { copy as toClipboard } from "../lib/clipboard";
import { label } from "../components/status";
import { query } from "../lib/query";
import { reason } from "../lib/provider";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  topic?: string;
  today?: string;
  call?: Transport;
  copy?: (text: string) => Promise<void>;
}

export default function Exam(props: Props) {
  const call = () => props.call ?? transport;
  const copy = () => props.copy ?? toClipboard;
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);

  const [prompt, setPrompt] = createSignal("");
  const [copied, setCopied] = createSignal(false);
  const [manual, setManual] = createSignal(false);
  const [answer, setAnswer] = createSignal("");
  const [parsed, setParsed] = createSignal<VerdictView | null>(null);
  const [broken, setBroken] = createSignal(false);
  const [applied, setApplied] = createSignal<ApplyVerdictOut | null>(null);
  const [built, setBuilt] = createSignal(false);
  const [asking, setAsking] = createSignal(false);
  const [refused, setRefused] = createSignal("");

  onMount(() => {
    void (async () => {
      const out = await call()("prompt", { bundle: program(), topic: id() });
      setPrompt(out.text);
    })();
    void (async () => {
      const out = await call()("provider", {
        save: null,
        key: null,
        forget: false,
        check: false,
      });
      setBuilt(out.provider.enabled);
    })();
  });

  const onCopy = () => {
    void (async () => {
      try {
        await copy()(prompt());
        setCopied(true);
      } catch {
        setManual(true);
      }
    })();
  };

  const onParse = () => {
    void (async () => {
      setApplied(null);
      try {
        setParsed(await call()("parse_verdict", { bundle: program(), topic: id(), text: answer() }));
        setBroken(false);
      } catch {
        setParsed(null);
        setBroken(true);
      }
    })();
  };

  const onAsk = () => {
    void (async () => {
      setAsking(true);
      setRefused("");
      try {
        const out = await call()("examine", { bundle: program(), topic: id() });
        setAnswer(out.text);
        onParse();
      } catch (error) {
        setRefused(reason(error, props.text));
      }
      setAsking(false);
    })();
  };

  const onApply = () => {
    void (async () => {
      setApplied(
        await call()("apply_verdict", {
          bundle: program(),
          topic: id(),
          text: answer(),
          today: today(),
        }),
      );
    })();
  };

  const review = () =>
    `/${props.locale}/review/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  return (
    <article>
      <p>
        <button type="button" data-copy onClick={onCopy}>
          {props.text.exam.copy}
        </button>
        <Show when={copied()}>
          <span data-copied>{props.text.exam.copied}</span>
        </Show>
        <Show when={manual()}>
          <span data-manual>{props.text.exam.copyManually}</span>
        </Show>
      </p>
      <pre data-prompt-text>{prompt()}</pre>

      <label>
        {props.text.exam.paste}
        <textarea
          data-verdict-input
          rows={12}
          value={answer()}
          onInput={(event) => setAnswer(event.currentTarget.value)}
        />
      </label>
      <p>
        <button type="button" data-parse onClick={onParse}>
          {props.text.exam.parse}
        </button>
        <Show when={built()}>
          <button type="button" data-ask disabled={asking()} onClick={onAsk}>
            {asking() ? props.text.exam.asking : props.text.exam.ask}
          </button>
        </Show>
      </p>
      <Show when={refused() !== ""}>
        <p data-ask-failed>{refused()}</p>
      </Show>

      <Show when={broken()}>
        <p data-broken>{props.text.exam.broken}</p>
      </Show>
      <Show when={parsed()}>
        {(verdict) => (
          <>
            <Parsed text={props.text} verdict={verdict()} />
            <p>
              <button type="button" data-apply onClick={onApply}>
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
            <Show when={out().split_suggested}>
              <p data-split>{props.text.exam.split}</p>
            </Show>
            <a href={review()} data-review-link>
              {props.text.exam.review}
            </a>
          </section>
        )}
      </Show>
    </article>
  );
}

import { Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AnswerView, StageIn } from "../../ipc";
import { copy as toClipboard } from "../../lib/clipboard";
import type { Transport } from "../../lib/ipc";
import { type Refused, refusal } from "../../lib/refusal";
import { toast } from "../../lib/toast";
import Refusal from "../generate/Refusal";
import type { Copier } from "../rich/Snip";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  at: StageIn;
  answers: () => AnswerView[];
  flush: () => Promise<void>;
  locked: boolean;
  copy?: Copier;
  done: () => void;
}

export default function Paste(props: Props) {
  const stage = () => props.text.stage;
  const [busy, setBusy] = createSignal(false);
  const [shown, setShown] = createSignal("");
  const [pasted, setPasted] = createSignal("");
  const [refused, setRefused] = createSignal<Refused | null>(null);

  const refuse = (reason: string) => setRefused({ reason, settings: false, failed: false });

  const guarded = async (work: () => Promise<void>) => {
    setBusy(true);
    setRefused(null);
    try {
      await work();
    } catch (failure) {
      setRefused(refusal(failure, props.text));
    } finally {
      setBusy(false);
    }
  };

  const hand = async (prompt: string) => {
    try {
      await (props.copy ?? toClipboard)(prompt);
      setShown("");
      toast("ok", stage().prompted);
    } catch {
      setShown(prompt);
      toast("warn", stage().promptManual);
    }
  };

  const take = () => {
    const answers = props.answers();
    if (answers.every((answer) => answer.text.trim() === "")) {
      refuse(stage().blank);
      return;
    }
    void guarded(async () => {
      await props.flush();
      const out = await props.call("exam_prompt", { ...props.at, answers });
      await hand(out.prompt);
    });
  };

  const apply = () => {
    const text = pasted();
    if (text.trim() === "") {
      refuse(stage().pasteBlank);
      return;
    }
    void guarded(async () => {
      const sat = await props.call("exam_paste", { ...props.at, text });
      setPasted("");
      toast(sat.passed ? "ok" : "info", sat.passed ? stage().passed : stage().graded);
      props.done();
    });
  };

  const idle = () => !busy() && !props.locked;

  return (
    <details data-copypaste>
      <summary>{stage().elsewhere}</summary>
      <p>{stage().elsewhereLead}</p>
      <button type="button" data-prompt disabled={!idle()} onClick={take}>
        {stage().prompt}
      </button>
      <Show when={shown()}>
        {(prompt) => <textarea data-prompt-text readOnly rows="6" aria-label={stage().prompt} value={prompt()} />}
      </Show>
      <label>
        {stage().paste}
        <textarea
          data-paste
          rows="6"
          readOnly={busy()}
          value={pasted()}
          onInput={(event) => setPasted(event.currentTarget.value)}
        />
      </label>
      <button type="button" data-apply disabled={!idle()} onClick={apply}>
        {stage().apply}
      </button>
      <Refusal text={props.text} locale={props.locale} refused={refused()} />
    </details>
  );
}

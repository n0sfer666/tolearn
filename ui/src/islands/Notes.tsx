import { Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { StampView } from "../ipc";
import { query } from "../lib/query";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  topic?: string;
  call?: Transport;
}

interface Conflict {
  theirs: string;
  ours: string;
}

export default function Notes(props: Props) {
  const call = () => props.call ?? transport;
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");

  const [body, setBody] = createSignal("");
  const [stamp, setStamp] = createSignal<StampView | null>(null);
  const [saved, setSaved] = createSignal(false);
  const [conflict, setConflict] = createSignal<Conflict | null>(null);

  const reread = () => {
    void (async () => {
      const out = await call()("note", { bundle: program(), topic: id(), directory: null });
      setBody(out.body);
      setStamp(out.stamp);
      setConflict(null);
      setSaved(false);
    })();
  };

  onMount(reread);

  const onSave = () => {
    void (async () => {
      const out = await call()("save_note", {
        bundle: program(),
        topic: id(),
        body: body(),
        directory: null,
        stamp: stamp(),
      });
      if (out.saved === false) {
        setConflict({ theirs: out.theirs ?? "", ours: body() });
        return;
      }
      setStamp(out.stamp);
      setConflict(null);
      setSaved(true);
    })();
  };

  const onInput = (text: string) => {
    setBody(text);
    setSaved(false);
  };

  const back = () =>
    `/${props.locale}/topic/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  return (
    <article>
      <textarea
        data-note
        rows={20}
        aria-label={props.text.topic.notes}
        value={body()}
        onInput={(event) => onInput(event.currentTarget.value)}
      />
      <p>
        <button type="button" data-save onClick={onSave}>
          {props.text.notes.save}
        </button>
        <button type="button" data-reread onClick={reread}>
          {props.text.notes.reread}
        </button>
        <Show when={saved()}>
          <span data-saved>{props.text.notes.saved}</span>
        </Show>
      </p>
      <Show when={conflict()}>
        {(both) => (
          <section data-conflict>
            <h2>{props.text.notes.conflict}</h2>
            <p>{props.text.notes.conflictLead}</p>
            <h3>{props.text.notes.theirs}</h3>
            <pre data-theirs>{both().theirs}</pre>
            <h3>{props.text.notes.ours}</h3>
            <pre data-ours>{both().ours}</pre>
          </section>
        )}
      </Show>
      <a href={back()} data-topic-link>
        {props.text.practice.open}
      </a>
    </article>
  );
}

import { For, Show, createSignal, onMount } from "solid-js";

import Header from "../components/topic/Header";
import Materials from "../components/topic/Materials";
import Practice from "../components/topic/Practice";
import type { Status } from "../components/status";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { TopicOut } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  topic?: string;
  today?: string;
  call?: Transport;
}

function query(name: string): string {
  return new URLSearchParams(location.search).get(name) ?? "";
}

export default function Topic(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");

  const [topic, setTopic] = createSignal<TopicOut | null>(null);
  const [gone, setGone] = createSignal(false);

  onMount(() => {
    if (program() === "" || id() === "") {
      setGone(true);
      return;
    }
    void (async () => {
      try {
        setTopic(await call()("topic", { bundle: program(), topic: id(), today: today() }));
      } catch {
        setGone(true);
      }
    })();
  });

  const pick = (status: Status) => {
    void (async () => {
      const out = await call()("set_status", {
        bundle: program(),
        topic: id(),
        status,
        today: today(),
      });
      setTopic((was) => (was === null ? was : { ...was, status: out.status }));
    })();
  };

  const practice = () => `/${props.locale}/practice/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  const notes = () => `/${props.locale}/notes/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  return (
    <Show
      when={topic()}
      fallback={
        <Show when={gone()}>
          <p data-empty>
            {props.text.topic.none}{" "}
            <a href={`/${props.locale}/`}>{props.text.nav.programs}</a>
          </p>
        </Show>
      }
    >
      {(view) => (
        <article>
          <Header text={props.text} topic={view()} onPick={pick} />

          <section data-section="outcomes">
            <h2>{props.text.topic.outcomes}</h2>
            <ul>
              <For each={view().outcomes}>{(outcome) => <li>{outcome}</li>}</For>
            </ul>
          </section>

          <Show when={view().misconceptions.length > 0}>
            <section data-section="misconceptions">
              <h2>{props.text.topic.misconceptions}</h2>
              <ul>
                <For each={view().misconceptions}>{(item) => <li>{item}</li>}</For>
              </ul>
            </section>
          </Show>

          <Show when={view().materials.length > 0}>
            <section data-section="materials">
              <Materials text={props.text} materials={view().materials} />
            </section>
          </Show>

          <section data-section="practice">
            <h2>{props.text.topic.practice}</h2>
            <Practice text={props.text} practice={view().practice} />
            <a href={practice()} data-practice-link>
              {props.text.practice.open}
            </a>
          </section>

          <Show when={view().questions.length > 0}>
            <section data-section="questions">
              <h2>{props.text.topic.questions}</h2>
              <ul>
                <For each={view().questions}>
                  {(question) => (
                    <li data-question={question.id}>
                      <span data-kind>{question.kind}</span>
                      <span>{question.text}</span>
                    </li>
                  )}
                </For>
              </ul>
            </section>
          </Show>

          <section data-section="exam">
            <h2>{props.text.topic.exam}</h2>
            <p>
              {props.text.topic.focus}: {view().exam.focus}
            </p>
            <Show when={view().exam.artifact_required}>
              <p data-artifact>{props.text.topic.artifact}</p>
            </Show>
          </section>

          <section data-section="notes">
            <h2>{props.text.topic.notes}</h2>
            <a href={notes()}>{props.text.topic.notes}</a>
          </section>
        </article>
      )}
    </Show>
  );
}

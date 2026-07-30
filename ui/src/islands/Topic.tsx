import { For, Show, createSignal, onMount } from "solid-js";

import Header from "../components/topic/Header";
import Materials from "../components/topic/Materials";
import Notes from "./Notes";
import type { Status } from "../components/status";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { TopicOut } from "../ipc";
import { name } from "../lib/name";
import { remember, shown } from "../lib/panel";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

const NOTE_PANEL = "tolearn.note-open";

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
  const [open, setOpen] = createSignal(shown(NOTE_PANEL, false));

  const flip = (next: boolean) => {
    setOpen(next);
    remember(NOTE_PANEL, next);
  };

  const refresh = () => {
    void (async () => {
      try {
        const out = await call()("topic", { bundle: program(), topic: id(), today: today() });
        setTopic(out);
        name("topic", id(), out.title);
        name("program", program(), out.program);
      } catch {
        setGone(true);
      }
    })();
  };

  onMount(() => {
    if (program() === "" || id() === "") {
      setGone(true);
      return;
    }
    refresh();
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

  const where = () =>
    `?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;
  const practice = () => `/${props.locale}/practice/${where()}`;
  const exam = () => `/${props.locale}/exam/${where()}`;

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

          <Show when={view().materials.length > 0}>
            <section data-section="materials">
              <Materials
                text={props.text}
                locale={props.locale}
                bundle={program()}
                topic={id()}
                materials={view().materials}
                call={props.call}
                onSaved={refresh}
              />
            </section>
          </Show>

          <nav class="row" data-tools aria-label={props.text.nav.sections}>
            <a href={practice()} data-practice-link>
              {props.text.topic.practice}
            </a>
            <a href={exam()} data-exam-link>
              {props.text.topic.exam}
            </a>
          </nav>

          <section data-section="outcomes">
            <h2>{props.text.topic.outcomes}</h2>
            <ul>
              <For each={view().outcomes}>{(outcome) => <li>{outcome}</li>}</For>
            </ul>
          </section>

          <details data-section="exam">
            <summary>{props.text.topic.exam}</summary>
            <p>
              {props.text.topic.focus}: {view().exam.focus}
            </p>
            <Show when={view().exam.artifact_required}>
              <p data-artifact>{props.text.topic.artifact}</p>
            </Show>
          </details>

          <Show when={view().questions.length > 0}>
            <details data-section="questions">
              <summary>{props.text.topic.questions}</summary>
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
            </details>
          </Show>

          <Show when={view().misconceptions.length > 0}>
            <details data-section="misconceptions">
              <summary>{props.text.topic.misconceptions}</summary>
              <ul>
                <For each={view().misconceptions}>{(item) => <li>{item}</li>}</For>
              </ul>
            </details>
          </Show>

          <button type="button" data-note-fab aria-expanded={open()} onClick={() => flip(!open())}>
            {props.text.topic.notes}
          </button>

          <Show when={open()}>
            <aside data-note-panel aria-label={props.text.topic.notes}>
              <header>
                <h2>{props.text.topic.notes}</h2>
                <button type="button" data-note-close onClick={() => flip(false)}>
                  {props.text.notes.hide}
                </button>
              </header>
              <Notes
                text={props.text}
                locale={props.locale}
                program={program()}
                topic={id()}
                standalone={false}
                call={props.call}
              />
            </aside>
          </Show>
        </article>
      )}
    </Show>
  );
}

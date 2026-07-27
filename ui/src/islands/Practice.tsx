import { For, Show, createSignal, onMount } from "solid-js";

import CheckItem, { type Ran } from "../components/practice/CheckItem";
import type { CheckView, TopicOut } from "../ipc";
import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
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

export default function Practice(props: Props) {
  const call = () => props.call ?? transport;
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");

  const [topic, setTopic] = createSignal<TopicOut | null>(null);
  const [ran, setRan] = createSignal<Record<string, Ran>>({});
  const [running, setRunning] = createSignal<string | null>(null);
  const [done, setDone] = createSignal<Record<string, boolean>>({});

  onMount(() => {
    void (async () => {
      setTopic(await call()("topic", { bundle: program(), topic: id(), today: today() }));
    })();
  });

  const today = () => props.today ?? new Date().toISOString().slice(0, 10);

  const put = (check: string, value: Ran) => setRan((was) => ({ ...was, [check]: value }));

  const onRun = (check: string) => {
    setRunning(check);
    void (async () => {
      try {
        const out = await call()("run_check", { bundle: program(), topic: id(), check });
        put(check, { out, failed: false });
      } catch {
        put(check, { out: null, failed: true });
      }
      setRunning(null);
    })();
  };

  const onDone = (check: string) =>
    setDone((was) => ({ ...was, [check]: was[check] !== true }));

  const list = (checks: CheckView[]) => (
    <ul>
      <For each={checks}>
        {(check) => (
          <CheckItem
            text={props.text}
            check={check}
            ran={ran()[check.id]}
            running={running() === check.id}
            done={done()[check.id] === true}
            onRun={onRun}
            onDone={onDone}
          />
        )}
      </For>
    </ul>
  );

  const back = () =>
    `/${props.locale}/topic/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(id())}`;

  return (
    <Show when={topic()}>
      {(view) => (
        <article>
          <p data-task>{view().practice.task}</p>
          <p data-deliverable>
            {props.text.topic.deliverable}: {view().practice.deliverable}
          </p>
          <p data-box>
            {props.text.topic.timeBox}: {view().practice.time_box_min} {props.text.topic.minutes}
            <Show when={view().practice.smoke_checked}>
              <span data-smoke>{props.text.topic.smoke}</span>
            </Show>
          </p>
          <section data-constraints>
            <h2>{props.text.topic.constraints}</h2>
            {list(view().practice.constraints)}
          </section>
          <section data-acceptance>
            <h2>{props.text.topic.acceptance}</h2>
            {list(view().practice.acceptance)}
          </section>
          <Show when={view().practice.fallback}>
            {(hint) => (
              <details>
                <summary>{props.text.topic.hint}</summary>
                <p>{hint()}</p>
              </details>
            )}
          </Show>
          <a href={back()} data-topic-link>
            {props.text.practice.open}
          </a>
        </article>
      )}
    </Show>
  );
}

import { For, Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { DueView } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: string;
  today?: string;
  call?: Transport;
}

export default function Queue(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);

  const [due, setDue] = createSignal<DueView[]>([]);
  const [busy, setBusy] = createSignal("");
  const [refused, setRefused] = createSignal("");

  const list = async () => {
    const answer = await call()("queue", { today: today() });
    setDue(answer.due);
  };

  const overdue = () => due().filter((item) => item.overdue);
  const now = () => due().filter((item) => !item.overdue);

  const href = (item: DueView) =>
    `/${props.locale}/topic/?program=${encodeURIComponent(item.bundle)}&topic=${encodeURIComponent(item.topic)}`;

  const onRepeat = (item: DueView) => {
    void (async () => {
      setBusy(item.topic);
      setRefused("");
      try {
        await call()("repeat", { bundle: item.bundle, topic: item.topic, today: today() });
        await list();
      } catch (error) {
        setRefused(said(error, props.text.queue.failed));
      }
      setBusy("");
    })();
  };

  onMount(() => void list());

  const group = (items: DueView[], mark: "overdue" | "today", heading: string) => (
    <Show when={items.length > 0}>
      <section data-overdue={mark === "overdue" ? "" : undefined} data-today={mark === "today" ? "" : undefined}>
        <h3>{heading}</h3>
        <ul>
          <For each={items}>
            {(item) => (
              <li data-due={item.topic}>
                <a href={href(item)}>{item.topic_title}</a>
                <span data-program>{item.title}</span>
                <time datetime={item.due}>{item.due}</time>
                <button type="button" data-repeat disabled={busy() === item.topic} onClick={() => onRepeat(item)}>
                  {busy() === item.topic ? props.text.queue.repeating : props.text.queue.repeat}
                </button>
              </li>
            )}
          </For>
        </ul>
      </section>
    </Show>
  );

  return (
    <section>
      <Show when={due().length > 0} fallback={<p data-empty>{props.text.queue.empty}</p>}>
        {group(overdue(), "overdue", props.text.queue.overdue)}
        {group(now(), "today", props.text.queue.today)}
      </Show>
      <Show when={refused() !== ""}>
        <p data-repeat-failed role="alert">
          {refused()}
        </p>
      </Show>
    </section>
  );
}

function said(error: unknown, fallback: string): string {
  if (error instanceof Object && "message" in error && typeof error.message === "string") {
    return error.message;
  }
  return fallback;
}

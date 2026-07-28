import { For, Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { AgingView, ExpiredView, StaleOut } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: string;
  path?: string;
  today?: string;
  call?: Transport;
}

export default function Stale(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const path = () => props.path ?? new URLSearchParams(location.search).get("program") ?? "";

  const [taken, setTaken] = createSignal<StaleOut | null>(null);

  onMount(() => {
    void (async () => {
      setTaken(await call()("stale", { bundle: path(), today: today() }));
    })();
  });

  const topics = () => taken()?.topics ?? [];
  const materials = () => taken()?.materials ?? [];
  const quiet = () => taken() !== null && topics().length === 0 && materials().length === 0;

  const href = (topic: string) =>
    `/${props.locale}/topic/?program=${encodeURIComponent(path())}&topic=${encodeURIComponent(topic)}`;

  const expired = (item: ExpiredView) => (
    <li data-expired={item.topic}>
      <a href={href(item.topic)}>{item.title}</a>
      <span data-verified>
        {props.text.stale.verified}: <time datetime={item.verified_at}>{item.verified_at}</time>
      </span>
      <span data-since>
        {props.text.stale.expired}: <time datetime={item.expired_at}>{item.expired_at}</time>
      </span>
    </li>
  );

  const aging = (item: AgingView) => (
    <li data-aging={item.title}>
      <a href={item.url}>{item.title}</a>
      <a data-topic href={href(item.topic)}>
        {item.topic_title}
      </a>
      <Show when={item.stale}>
        <span data-flagged>{props.text.stale.flagged}</span>
      </Show>
      <Show when={item.pin === "unknown"}>
        <span data-pin>
          {props.text.stale.pinUnknown}: {item.covers_version}
        </span>
      </Show>
      <Show when={item.delta !== null}>
        <p data-delta>{item.delta}</p>
      </Show>
    </li>
  );

  return (
    <Show when={taken()}>
      <Show when={!quiet()} fallback={<p data-empty>{props.text.stale.empty}</p>}>
        <Show when={topics().length > 0}>
          <section data-topics>
            <h3>{props.text.stale.topics}</h3>
            <ul>
              <For each={topics()}>{expired}</For>
            </ul>
          </section>
        </Show>
        <Show when={materials().length > 0}>
          <section data-materials>
            <h3>{props.text.stale.materials}</h3>
            <ul>
              <For each={materials()}>{aging}</For>
            </ul>
          </section>
        </Show>
      </Show>
    </Show>
  );
}

import { For, Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { ReadOfflineOut } from "../ipc";
import { query } from "../lib/query";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  url?: string;
  call?: Transport;
}

export default function Read(props: Props) {
  const call = () => props.call ?? transport;
  const url = () => props.url ?? query("url");

  const [reading, setReading] = createSignal<ReadOfflineOut | null>(null);
  const [gone, setGone] = createSignal(false);

  onMount(() => {
    if (url() === "") {
      setGone(true);
      return;
    }
    void (async () => {
      try {
        setReading(await call()("read_offline", { url: url() }));
      } catch {
        setGone(true);
      }
    })();
  });

  const paragraphs = () =>
    (reading()?.text ?? "")
      .split(/\n{2,}/)
      .map((part) => part.trim())
      .filter((part) => part !== "");

  const source = () => (
    <Show when={url() !== ""}>
      <a data-source href={url()}>
        {props.text.offline.source}
      </a>
    </Show>
  );

  return (
    <Show
      when={reading()}
      fallback={
        <Show when={gone()} fallback={<p data-loading>{props.text.offline.reading}</p>}>
          <p data-empty>
            {url() === "" ? props.text.offline.none : props.text.offline.absent} {source()}
          </p>
        </Show>
      }
    >
      {(out) => (
        <article data-reading={out().kind}>
          <Show when={out().title !== ""}>
            <h2>{out().title}</h2>
          </Show>
          <Show
            when={out().extracted}
            fallback={
              <p data-file>
                {props.text.offline.file}: <code>{out().path}</code>
              </p>
            }
          >
            <For each={paragraphs()}>{(part) => <p>{part}</p>}</For>
          </Show>
          {source()}
        </article>
      )}
    </Show>
  );
}

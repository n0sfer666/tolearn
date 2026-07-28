import { For, Show, createSignal, onMount } from "solid-js";

import Questions from "../components/review/Questions";
import type { Dictionary } from "../i18n/ru";
import type { ReviewOut } from "../ipc";
import { copy as toClipboard } from "../lib/clipboard";
import { query } from "../lib/query";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { verdict } from "../components/review/labels";

interface Props {
  text: Dictionary;
  program?: string;
  topic?: string;
  call?: Transport;
  copy?: (text: string) => Promise<void>;
}

export default function Review(props: Props) {
  const call = () => props.call ?? transport;
  const copy = () => props.copy ?? toClipboard;
  const program = () => props.program ?? query("program");
  const id = () => props.topic ?? query("topic");

  const [seen, setSeen] = createSignal<ReviewOut | null>(null);
  const [copied, setCopied] = createSignal(false);
  const [manual, setManual] = createSignal(false);

  onMount(() => {
    void (async () => {
      setSeen(await call()("review", { bundle: program(), topic: id() }));
    })();
  });

  const onCopy = (request: string) => {
    void (async () => {
      try {
        await copy()(request);
        setCopied(true);
      } catch {
        setManual(true);
      }
    })();
  };

  return (
    <Show when={seen()}>
      {(out) => (
        <article data-review>
          <h3>{out().title}</h3>
          <Questions text={props.text} questions={out().questions} />

          <Show when={out().loose.length > 0}>
            <p data-loose>
              {props.text.review.loose}: {out().loose.join(", ")}
            </p>
          </Show>

          <Show when={out().last} fallback={<p data-none>{props.text.review.none}</p>}>
            {(last) => (
              <p data-last>
                {props.text.review.last}: {last().at} — {verdict(props.text, last().verdict)}
              </p>
            )}
          </Show>

          <Show when={out().history.length > 0}>
            <ul data-history>
              <For each={out().history}>
                {(past) => (
                  <li>
                    {past.at} — {verdict(props.text, past.verdict)}
                  </li>
                )}
              </For>
            </ul>
          </Show>

          <Show when={out().split_suggested}>
            <section data-split>
              <p>{props.text.review.split}</p>
              <p>{props.text.review.splitLead}</p>
              <p>
                <button type="button" data-copy onClick={() => onCopy(out().split_request)}>
                  {props.text.review.copy}
                </button>
                <Show when={copied()}>
                  <span data-copied>{props.text.review.copied}</span>
                </Show>
                <Show when={manual()}>
                  <span data-manual>{props.text.review.copyManually}</span>
                </Show>
              </p>
              <pre data-request>{out().split_request}</pre>
            </section>
          </Show>
        </article>
      )}
    </Show>
  );
}

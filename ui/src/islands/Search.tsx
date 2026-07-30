import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { HitView } from "../ipc";
import type { Locale } from "../i18n";
import { opened } from "../lib/query";
import { quiet } from "../lib/ipc";
import { toast } from "../lib/toast";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  call?: Transport;
}

export default function Search(props: Props) {
  const call = () => props.call ?? quiet;
  const program = () => props.program ?? opened();

  const [asked, setAsked] = createSignal("");
  const [hits, setHits] = createSignal<HitView[]>([]);
  const [ran, setRan] = createSignal(false);

  const find = () => {
    void (async () => {
      try {
        const out = await call()("search", {
          bundle: program(),
          query: asked(),
          directory: null,
          limit: 20,
        });
        setHits(out.hits);
        setRan(true);
      } catch {
        setHits([]);
        setRan(false);
        toast("error", props.text.search.failed);
      }
    })();
  };

  const kind = (hit: HitView) => {
    if (hit.kind === "note") return props.text.search.note;
    if (hit.kind === "material") return props.text.search.material;
    return props.text.search.topic;
  };

  const href = (hit: HitView) => {
    const where = hit.kind === "note" ? "notes" : "topic";
    return `/${props.locale}/${where}/?program=${encodeURIComponent(program())}&topic=${encodeURIComponent(hit.topic)}`;
  };

  return (
    <Show
      when={program() !== ""}
      fallback={
        <p data-empty>
          {props.text.program.none} <a href={`/${props.locale}/`}>{props.text.nav.programs}</a>
        </p>
      }
    >
      <article>
        <form
          class="row"
          onSubmit={(event) => {
            event.preventDefault();
            find();
          }}
        >
          <input
            type="search"
            data-query
            autofocus
            aria-label={props.text.search.placeholder}
            placeholder={props.text.search.placeholder}
            value={asked()}
            onInput={(event) => setAsked(event.currentTarget.value)}
          />
          <button type="submit" data-find>
            {props.text.search.find}
          </button>
        </form>
        <Show when={ran() && hits().length === 0}>
          <p data-nothing>{props.text.search.nothing}</p>
        </Show>
        <ul data-hits>
          <For each={hits()}>
            {(hit) => (
              <li data-hit={hit.kind}>
                <a href={href(hit)}>{hit.title}</a>
                <span data-kind>{kind(hit)}</span>
                <p data-snippet>{hit.snippet}</p>
              </li>
            )}
          </For>
        </ul>
      </article>
    </Show>
  );
}

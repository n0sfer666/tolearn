import { For, Show, createMemo, createSignal } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { HitView } from "../ipc";
import type { Locale } from "../i18n";
import { quiet } from "../lib/ipc";
import { stageHref } from "../lib/links";
import { toast } from "../lib/toast";
import type { Transport } from "../lib/ipc";

const LIMIT = 20;

interface Props {
  text: Dictionary;
  locale: Locale;
  call?: Transport;
}

export default function Search(props: Props) {
  const call = () => props.call ?? quiet;
  const [asked, setAsked] = createSignal("");
  const [hits, setHits] = createSignal<HitView[]>([]);
  const [ran, setRan] = createSignal(false);
  const [roots, setRoots] = createSignal<ReadonlyMap<string, string>>(new Map());
  const [listed, setListed] = createSignal(false);

  const list = () => {
    if (listed()) return;
    setListed(true);
    void (async () => {
      try {
        const out = await call()("library", {});
        setRoots(new Map(out.programs.map((shelf) => [shelf.uuid, shelf.title])));
      } catch {
        setRoots(new Map());
      }
    })();
  };

  const find = () => {
    list();
    void (async () => {
      try {
        const out = await call()("search", { query: asked(), limit: LIMIT });
        setHits(out.hits);
        setRan(true);
      } catch {
        setHits([]);
        setRan(false);
        toast("error", props.text.search.failed);
      }
    })();
  };

  const kind = (hit: HitView) => (hit.kind === "stage" ? props.text.search.stage : props.text.search.block);

  const tally = () => {
    const count = hits().length;
    const template = count === LIMIT ? props.text.search.shown : props.text.search.found;
    return template.replace("{n}", String(count));
  };

  const nested = (hit: HitView) => hit.node !== hit.program;

  const where = (hit: HitView) => {
    const program = roots().get(hit.program) ?? "";
    return { program, node: nested(hit) || program === "" ? hit.node_title : "" };
  };

  const href = (hit: HitView) => {
    const stage = stageHref(props.locale, hit.program, hit.node, hit.stage);
    return hit.block === "" ? stage : `${stage}#${hit.block}`;
  };

  return (
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
      <div data-tally aria-live="polite">
        <Show when={ran() && hits().length > 0}>
          <p data-found>{tally()}</p>
        </Show>
        <Show when={ran() && hits().length === 0}>
          <p data-nothing>{props.text.search.nothing}</p>
        </Show>
      </div>
      <ul data-hits>
        <For each={hits()}>
          {(hit) => {
            const place = createMemo(() => where(hit));
            return (
              <li data-hit={hit.kind}>
                <a href={href(hit)}>{hit.title}</a>
                <span data-kind>{kind(hit)}</span>
                <Show when={place().program !== "" || place().node !== ""}>
                  <p data-where>
                    <Show when={place().program !== ""}>
                      <span data-program>{place().program}</span>
                    </Show>
                    <Show when={place().node !== ""}>
                      <span data-node>{place().node}</span>
                    </Show>
                  </p>
                </Show>
                <Show when={hit.snippet !== ""}>
                  <p data-snippet>{hit.snippet}</p>
                </Show>
              </li>
            );
          }}
        </For>
      </ul>
    </article>
  );
}

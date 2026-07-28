import { For, Show, createSignal, onMount } from "solid-js";

import type { Card, ImportOut, Merged } from "../ipc";
import { matches } from "../lib/filter";
import { drops as listen, pick as choose, pickArchive as chooseArchive, transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: {
    list: string;
    listLead: string;
    choose: string;
    chooseArchive: string;
    drop: string;
    importing: string;
    unreachable: string;
    refused: string;
    merged: string;
    kept: string;
    added: string;
    stale: string;
    orphaned: string;
    progress: string;
    filter: string;
  };
  today?: string;
  call?: Transport;
  pick?: () => Promise<string | null>;
  pickArchive?: () => Promise<string | null>;
  drops?: (handler: (paths: string[]) => void) => void;
}

export default function Programs(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);

  const [cards, setCards] = createSignal<Card[]>([]);
  const [report, setReport] = createSignal<Merged | null>(null);
  const [refused, setRefused] = createSignal<string[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [needle, setNeedle] = createSignal("");

  const list = async () => {
    const { programs } = await call()("programs", { today: today() });
    setCards(programs);
  };

  const accept = async (path: string) => {
    if (busy()) return;
    setBusy(true);
    setReport(null);
    setRefused([]);
    try {
      const done: ImportOut = await call()("import", { path, today: today() });
      if (!done.ok) {
        setRefused(done.violations.map((violation) => violation.message));
        return;
      }
      setReport(done.report);
      await list();
    } finally {
      setBusy(false);
    }
  };

  onMount(() => {
    void list();
    (props.drops ?? listen)((paths) => {
      const [first] = paths;
      if (first !== undefined) void accept(first);
    });
  });

  const shown = () => cards().filter((card) => matches(card.title, needle()));

  const take = async (open: () => Promise<string | null>) => {
    const chosen = await open();
    if (chosen !== null) await accept(chosen);
  };

  return (
    <section>
      <div class="intake">
        <p>{props.text.drop}</p>
        <button
          type="button"
          data-pick
          onClick={() => void take(props.pick ?? choose)}
          disabled={busy()}
        >
          {busy() ? props.text.importing : props.text.choose}
        </button>
        <button
          type="button"
          data-pick-archive
          onClick={() => void take(props.pickArchive ?? chooseArchive)}
          disabled={busy()}
        >
          {props.text.chooseArchive}
        </button>
      </div>

      <Show when={refused().length > 0}>
        <div data-refused role="alert">
          <h3>{props.text.refused}</h3>
          <ul>
            <For each={refused()}>{(message) => <li>{message}</li>}</For>
          </ul>
        </div>
      </Show>

      <Show when={report()}>
        {(merged) => (
          <div data-report>
            <h3>{props.text.merged}</h3>
            <ul>
              <li>
                {props.text.kept}: {merged().kept.length}
              </li>
              <li>
                {props.text.added}: {merged().added.length}
              </li>
              <li>
                {props.text.stale}: {merged().stale.length}
              </li>
              <li>
                {props.text.orphaned}: {merged().orphaned.length}
              </li>
            </ul>
            <For each={merged().stale}>
              {(topic) => (
                <p>
                  {topic.id}: {topic.changed.join(", ")}
                </p>
              )}
            </For>
          </div>
        )}
      </Show>

      <h2>{props.text.list}</h2>
      <Show when={cards().length > 0} fallback={<p>{props.text.listLead}</p>}>
        <input
          type="search"
          data-filter
          aria-label={props.text.filter}
          placeholder={props.text.filter}
          value={needle()}
          onInput={(event) => setNeedle(event.currentTarget.value)}
        />
        <ul class="cards">
          <For each={shown()}>
            {(card) => (
              <li data-program={card.id}>
                <a href={`/program/?program=${encodeURIComponent(card.path)}`}>{card.title}</a>
                <Show when={card.tally} fallback={<p class="unreachable">{props.text.unreachable}</p>}>
                  {(tally) => (
                    <p>
                      {props.text.progress}: {tally().done} / {tally().total}
                    </p>
                  )}
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </section>
  );
}

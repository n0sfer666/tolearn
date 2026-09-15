import { For, Show, createSignal, onMount } from "solid-js";

import Shelf from "../components/reading/Shelf";
import type { Locale } from "../i18n";
import type { Dictionary } from "../i18n/ru";
import type { ImportPackageOut, RefusedView, ShelfView } from "../ipc";
import { matches } from "../lib/filter";
import { drops as listen, pickPackage, quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { query } from "../lib/query";
import { explain, toast } from "../lib/toast";
import { told } from "../lib/told";

type Words = Dictionary["programs"];

interface Props {
  text: Words;
  locale: Locale;
  call?: Transport;
  pick?: () => Promise<string | null>;
  drops?: (handler: (paths: string[]) => void) => void;
}

function refusals(text: Words): ReadonlyMap<string, string> {
  return new Map([
    ["package.v1", text.v1],
    ["package.folder", text.folder],
    ["package.foreign", text.foreign],
  ]);
}

function announce(text: Words, done: ImportPackageOut): void {
  if (done.copy_of === null) toast("ok", `${text.imported}: ${done.title}`);
  else toast("info", `${text.copied}: ${done.title}`);
}

export default function Programs(props: Props) {
  const call = () => props.call ?? quiet;

  const [shelves, setShelves] = createSignal<ShelfView[]>([]);
  const [broken, setBroken] = createSignal<RefusedView[]>([]);
  const [refused, setRefused] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [needle, setNeedle] = createSignal("");

  const list = async () => {
    try {
      const out = await call()("library", {});
      setShelves(out.programs);
      setBroken(out.refused);
    } catch (failure) {
      toast("error", explain(failure));
    }
  };

  const accept = async (path: string) => {
    if (busy()) return;
    setBusy(true);
    setRefused("");
    try {
      announce(props.text, await call()("import_package", { path }));
      await list();
    } catch (failure) {
      setRefused(told(failure, refusals(props.text)));
      toast("error", props.text.refused);
    } finally {
      setBusy(false);
    }
  };

  onMount(() => {
    toast("error", query("refused"));
    void list();
    (props.drops ?? listen)((paths) => {
      const [first] = paths;
      if (first !== undefined) void accept(first);
    });
  });

  const take = async () => {
    if (busy()) return;
    const chosen = await (props.pick ?? pickPackage)();
    if (chosen !== null) await accept(chosen);
  };

  const shown = () => shelves().filter((shelf) => matches(shelf.title, needle()));
  const failed = () => broken().filter((entry) => matches(entry.directory, needle()));

  return (
    <section>
      <Show when={shelves().length > 0}>
        <input
          type="search"
          data-filter
          aria-label={props.text.filter}
          placeholder={props.text.filter}
          value={needle()}
          onInput={(event) => setNeedle(event.currentTarget.value)}
        />
      </Show>
      <Show when={shelves().length + broken().length > 0}>
        <ul class="cards">
          <For each={shown()}>
            {(shelf) => <Shelf shelf={shelf} text={props.text} locale={props.locale} />}
          </For>
          <For each={failed()}>
            {(entry) => (
              <li data-broken={entry.directory}>
                <strong>{entry.directory}</strong>
                <p>
                  {props.text.broken}: {entry.message}
                </p>
              </li>
            )}
          </For>
        </ul>
      </Show>

      <div class="intake">
        <p>{props.text.drop}</p>
        <button type="button" data-pick data-action onClick={() => void take()} aria-disabled={busy()}>
          {busy() ? props.text.importing : props.text.choose}
        </button>
      </div>

      <Show when={refused() !== ""}>
        <div data-refused role="alert">
          <h3>{props.text.refused}</h3>
          <p>{refused()}</p>
        </div>
      </Show>
    </section>
  );
}

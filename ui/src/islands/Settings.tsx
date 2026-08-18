import { For, Show, createSignal, onMount } from "solid-js";

import Budget from "../components/settings/Budget";
import { LOCALES, type Locale, localized } from "../i18n";
import { THEMES } from "../components/themes";
import type { Dictionary } from "../i18n/ru";
import type { SettingsView } from "../ipc";
import { pick, transport } from "../lib/ipc";
import { toast } from "../lib/toast";
import type { Transport } from "../lib/ipc";
import { remember as keepLocale } from "../lib/locale";
import { remember } from "../lib/theme";

interface Props {
  text: Dictionary;
  locale: Locale;
  call?: Transport;
  choose?: () => Promise<string | null>;
  go?: (url: string) => void;
}

export default function Settings(props: Props) {
  const call = () => props.call ?? transport;
  const choose = () => props.choose ?? pick;
  const go = () => props.go ?? ((url: string) => location.assign(url));

  const [view, setView] = createSignal<SettingsView | null>(null);

  const took = (next: SettingsView) => {
    setView(next);
    remember(next.theme);
    keepLocale(next.locale);
  };

  onMount(() => {
    void (async () => {
      took(await call()("settings", { save: null }));
    })();
  });

  const store = async (change: Partial<SettingsView>): Promise<boolean> => {
    const current = view();
    if (current === null) return false;
    const next: SettingsView = { ...current, ...change };
    try {
      took(await call()("settings", { save: next }));
      toast("ok", props.text.settings.saved);
      return true;
    } catch {
      toast("error", props.text.settings.failed);
      return false;
    }
  };

  const onBudget = (value: string) => {
    const budget = Number.parseInt(value, 10);
    if (Number.isNaN(budget) || budget < 1) return;
    void store({ disk_budget_mb: budget });
  };

  const onDepth = (value: string) => {
    const depth = Number.parseInt(value, 10);
    if (Number.isNaN(depth) || depth < 0) return;
    void store({ history_depth: depth });
  };

  const onShare = (value: string) => {
    const share = Number.parseInt(value, 10);
    if (Number.isNaN(share) || share < 0 || share > 100) return;
    void store({ history_share_percent: share });
  };

  const onChoose = () => {
    void (async () => {
      const chosen = await choose()();
      if (chosen !== null) await store({ notes_directory: chosen });
    })();
  };

  const onLocale = (event: MouseEvent, other: Locale) => {
    event.preventDefault();
    void (async () => {
      if (await store({ locale: other })) go()(localized("/settings/", other));
    })();
  };

  return (
    <Show when={view()}>
      {(current) => (
        <article data-cards>
          <section>
            <h2>{props.text.settings.theme}</h2>
            <nav aria-label={props.text.theme.switch}>
              <For each={THEMES}>
                {(choice) => (
                  <button
                    type="button"
                    data-theme-choice={choice}
                    aria-pressed={current().theme === choice}
                    onClick={() => void store({ theme: choice })}
                  >
                    {props.text.theme[choice]}
                  </button>
                )}
              </For>
            </nav>
          </section>

          <section>
            <h2>{props.text.settings.budget}</h2>
            <Budget text={props.text} value={current().disk_budget_mb} onPick={onBudget} />
          </section>

          <section>
            <h2>{props.text.settings.notes}</h2>
            <p data-notes>{current().notes_directory ?? props.text.settings.notesOwn}</p>
            <button type="button" data-choose onClick={onChoose}>
              {props.text.settings.choose}
            </button>
            <Show when={current().notes_directory !== null}>
              <button
                type="button"
                data-reset
                onClick={() => void store({ notes_directory: null })}
              >
                {props.text.settings.reset}
              </button>
            </Show>
          </section>

          <section>
            <h2>{props.text.settings.language}</h2>
            <nav aria-label={props.text.language.switch}>
              <For each={LOCALES}>
                {(other) => (
                  <a
                    href={localized("/settings/", other)}
                    data-locale={other}
                    hreflang={other}
                    lang={other}
                    aria-current={current().locale === other ? "true" : undefined}
                    onClick={(event) => onLocale(event, other)}
                  >
                    {props.text.language[other]}
                  </a>
                )}
              </For>
            </nav>
          </section>

          <section>
            <h2>{props.text.settings.history}</h2>
            <p>{props.text.settings.historyLead}</p>
            <label>
              {props.text.settings.historyDepth}
              <input
                data-history-depth
                type="number"
                min="0"
                value={current().history_depth}
                onChange={(event) => onDepth(event.currentTarget.value)}
              />
            </label>
            <label>
              {props.text.settings.historyShare}
              <input
                data-history-share
                type="number"
                min="0"
                max="100"
                value={current().history_share_percent}
                onChange={(event) => onShare(event.currentTarget.value)}
              />
            </label>
          </section>
        </article>
      )}
    </Show>
  );
}

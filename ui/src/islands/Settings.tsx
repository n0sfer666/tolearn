import { For, Show, createSignal, onMount } from "solid-js";

import { LOCALES, type Locale, localized } from "../i18n";
import { THEMES } from "../components/themes";
import type { Dictionary } from "../i18n/ru";
import type { SettingsView } from "../ipc";
import { pick, transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { remember } from "../lib/theme";

interface Props {
  text: Dictionary;
  locale: Locale;
  call?: Transport;
  choose?: () => Promise<string | null>;
}

export default function Settings(props: Props) {
  const call = () => props.call ?? transport;
  const choose = () => props.choose ?? pick;

  const [view, setView] = createSignal<SettingsView | null>(null);
  const [saved, setSaved] = createSignal(false);
  const [failed, setFailed] = createSignal(false);

  const took = (next: SettingsView) => {
    setView(next);
    remember(next.theme);
  };

  onMount(() => {
    void (async () => {
      took(await call()("settings", { save: null }));
    })();
  });

  const store = (change: Partial<SettingsView>) => {
    const current = view();
    if (current === null) return;
    const next: SettingsView = { ...current, ...change };
    void (async () => {
      try {
        took(await call()("settings", { save: next }));
        setFailed(false);
        setSaved(true);
      } catch {
        setFailed(true);
        setSaved(false);
      }
    })();
  };

  const onBudget = (value: string) => {
    const budget = Number.parseInt(value, 10);
    if (Number.isNaN(budget) || budget < 1) return;
    store({ disk_budget_mb: budget });
  };

  const onDepth = (value: string) => {
    const depth = Number.parseInt(value, 10);
    if (Number.isNaN(depth) || depth < 0) return;
    store({ history_depth: depth });
  };

  const onShare = (value: string) => {
    const share = Number.parseInt(value, 10);
    if (Number.isNaN(share) || share < 0 || share > 100) return;
    store({ history_share_percent: share });
  };

  const onChoose = () => {
    void (async () => {
      const chosen = await choose()();
      if (chosen !== null) store({ notes_directory: chosen });
    })();
  };

  return (
    <Show when={view()}>
      {(current) => (
        <article>
          <section>
            <h2>{props.text.settings.budget}</h2>
            <input
              data-budget
              type="number"
              min="1"
              aria-label={props.text.settings.budget}
              value={current().disk_budget_mb}
              onChange={(event) => onBudget(event.currentTarget.value)}
            />
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
                onClick={() => store({ notes_directory: null })}
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
                    onClick={() => store({ locale: other })}
                  >
                    {props.text.language[other]}
                  </a>
                )}
              </For>
            </nav>
          </section>

          <section>
            <h2>{props.text.settings.theme}</h2>
            <nav aria-label={props.text.theme.switch}>
              <For each={THEMES}>
                {(choice) => (
                  <button
                    type="button"
                    data-theme-choice={choice}
                    aria-pressed={current().theme === choice}
                    onClick={() => store({ theme: choice })}
                  >
                    {props.text.theme[choice]}
                  </button>
                )}
              </For>
            </nav>
          </section>

          <Show when={saved()}>
            <p data-saved>{props.text.settings.saved}</p>
          </Show>
          <Show when={failed()}>
            <p data-failed>{props.text.settings.failed}</p>
          </Show>
        </article>
      )}
    </Show>
  );
}

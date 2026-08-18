import { For, Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { StatsOut } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  path?: string;
  call?: Transport;
}

export default function Stats(props: Props) {
  const call = () => props.call ?? transport;
  const path = () => props.path ?? new URLSearchParams(location.search).get("program") ?? "";

  const [taken, setTaken] = createSignal<StatsOut | null>(null);

  onMount(() => {
    void (async () => {
      setTaken(await call()("stats", { bundle: path() }));
    })();
  });

  const percent = (share: number) =>
    new Intl.NumberFormat(props.locale, { style: "percent", maximumFractionDigits: 0 }).format(
      share,
    );

  return (
    <Show when={taken()}>
      {(stats) => (
        <section>
          <p data-attempts>
            {props.text.stats.attempts}: {stats().attempts}
          </p>
          <Show
            when={stats().enough}
            fallback={<p data-scarce>{props.text.stats.scarce}</p>}
          >
            <p data-hinted>
              {props.text.stats.hinted}: {stats().hinted} / {stats().attempts} ={" "}
              {percent(stats().hinted_share)}
            </p>
            <table data-kinds>
              <thead>
                <tr>
                  <td />
                  <th scope="col">{props.text.review.resultOk}</th>
                  <th scope="col">{props.text.review.resultPartial}</th>
                  <th scope="col">{props.text.review.resultMiss}</th>
                </tr>
              </thead>
              <tbody>
                <For each={stats().kinds}>
                  {(kind) => (
                    <tr data-kind={kind.kind}>
                      <th scope="row">{kind.kind}</th>
                      <td data-ok>{kind.ok}</td>
                      <td data-partial>{kind.partial}</td>
                      <td data-miss>{kind.miss}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
            <ul data-actions>
              <For each={stats().actions}>
                {(action) => (
                  <li data-action={action.action}>
                    {action.action}: {action.count}
                  </li>
                )}
              </For>
            </ul>
            <Show when={stats().streak.longest > 0}>
              <p data-streak>
                {props.text.stats.streak}: {stats().streak.longest} — {stats().streak.topic}
              </p>
            </Show>
          </Show>
          <Show when={stats().calibration.length > 0}>
            <section data-calibration>
              <h3>{props.text.stats.calibration}</h3>
              <ul>
                <For each={stats().calibration}>{(line) => <li>{line}</li>}</For>
              </ul>
            </section>
          </Show>
        </section>
      )}
    </Show>
  );
}

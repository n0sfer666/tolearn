import { Show, createSignal, onMount } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { Locale } from "../i18n";
import type { PlanOut } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: Locale;
  path?: string;
  today?: string;
  call?: Transport;
}

export default function Plan(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const path = () => props.path ?? new URLSearchParams(location.search).get("program") ?? "";

  const [ahead, setAhead] = createSignal<PlanOut | null>(null);

  onMount(() => {
    void (async () => {
      setAhead(await call()("plan", { bundle: path(), today: today() }));
    })();
  });

  const round = (value: number) =>
    new Intl.NumberFormat(props.locale, { maximumFractionDigits: 1 }).format(value);

  return (
    <Show when={ahead()}>
      {(plan) => (
        <section>
          <p data-norm>
            {props.text.plan.norm}: {round(plan().daily_hours)} {props.text.program.hours} (
            {plan().weekly_hours} {props.text.plan.weekly} ÷ 7)
          </p>
          <Show
            when={plan().left.max > 0}
            fallback={<p data-done>{props.text.plan.done}</p>}
          >
            <p data-formula>
              {plan().left.min}–{plan().left.max} {props.text.program.hours} ÷{" "}
              {round(plan().daily_hours)} {props.text.program.hours} = {plan().soonest.days}–
              {plan().latest.days} {props.text.plan.days}
            </p>
            <p data-finish>
              {props.text.plan.finish}: {plan().soonest.date} — {plan().latest.date}
            </p>
          </Show>
          <Show when={plan().unknown > 0}>
            <p data-unknown>
              {plan().unknown} {props.text.plan.unknown} — {props.text.plan.unknownLead}
            </p>
          </Show>
        </section>
      )}
    </Show>
  );
}

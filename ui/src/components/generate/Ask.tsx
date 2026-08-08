import { For, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { GenerateIn } from "../../ipc";

const LEVELS = ["beginner", "basics", "confident"] as const;

type Level = (typeof LEVELS)[number];

interface Props {
  text: Dictionary["generate"];
  onStart: (request: Omit<GenerateIn, "today">) => void;
  onClose: () => void;
}

const chosen = (value: string): Level => LEVELS.find((known) => known === value) ?? "basics";

const counted = (value: string) => {
  const number = Number(value);
  return Number.isFinite(number) && number > 0 ? Math.round(number) : 0;
};

export default function Ask(props: Props) {
  const [subject, setSubject] = createSignal("");
  const [level, setLevel] = createSignal<Level>("basics");
  const [weekly, setWeekly] = createSignal(6);
  const [weeks, setWeeks] = createSignal(10);

  const send = (event: Event) => {
    event.preventDefault();
    props.onStart({
      subject: subject().trim(),
      level: level(),
      weekly_hours: weekly(),
      weeks: weeks() > 0 ? weeks() : null,
    });
  };

  return (
    <form data-ask onSubmit={send}>
      <label>
        {props.text.subject}
        <textarea
          data-subject
          rows="3"
          required
          placeholder={props.text.subjectHint}
          value={subject()}
          onInput={(event) => setSubject(event.currentTarget.value)}
        />
      </label>
      <label>
        {props.text.level}
        <select
          data-level
          value={level()}
          onChange={(event) => setLevel(chosen(event.currentTarget.value))}
        >
          <For each={LEVELS}>{(known) => <option value={known}>{props.text[known]}</option>}</For>
        </select>
      </label>
      <label>
        {props.text.weekly}
        <input
          type="number"
          data-weekly
          min="1"
          value={weekly()}
          onInput={(event) => setWeekly(counted(event.currentTarget.value))}
        />
      </label>
      <label>
        {props.text.weeks}
        <input
          type="number"
          data-weeks
          min="0"
          value={weeks()}
          onInput={(event) => setWeeks(counted(event.currentTarget.value))}
        />
      </label>
      <div class="row">
        <button type="submit" data-start disabled={subject().trim() === "" || weekly() === 0}>
          {props.text.start}
        </button>
        <button type="button" data-ask-close onClick={props.onClose}>
          {props.text.cancel}
        </button>
      </div>
    </form>
  );
}

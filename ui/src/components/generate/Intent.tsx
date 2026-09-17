import { For } from "solid-js";

import { LOCALES } from "../../i18n";
import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  request: string;
  level: string;
  locale: string;
  locked: boolean;
  setRequest: (value: string) => void;
  setLevel: (value: string) => void;
  setLocale: (value: string) => void;
}

export default function Intent(props: Props) {
  return (
    <div data-intent>
      <label>
        {props.text.generate.request}
        <textarea
          data-request
          rows="3"
          required
          readOnly={props.locked || undefined}
          value={props.request}
          onInput={(event) => props.setRequest(event.currentTarget.value)}
        />
      </label>
      <label>
        {props.text.generate.level}
        <textarea
          data-level
          rows="2"
          required
          readOnly={props.locked || undefined}
          value={props.level}
          onInput={(event) => props.setLevel(event.currentTarget.value)}
        />
      </label>
      <label>
        {props.text.generate.tongue}
        <select
          data-tongue
          disabled={props.locked}
          value={props.locale}
          onInput={(event) => props.setLocale(event.currentTarget.value)}
        >
          <For each={LOCALES}>{(tongue) => <option value={tongue}>{props.text.language[tongue]}</option>}</For>
        </select>
      </label>
    </div>
  );
}

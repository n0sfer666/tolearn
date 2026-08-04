import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  value: string;
  stored: boolean;
  onChange: (next: string) => void;
  onForget: () => void;
}

export default function KeyField(props: Props) {
  return (
    <>
      <label>
        {props.text.provider.key}
        <input
          data-key
          type="password"
          value={props.value}
          onInput={(event) => props.onChange(event.currentTarget.value)}
        />
      </label>
      <p data-stored>
        {props.stored ? props.text.provider.keyStored : props.text.provider.keyEmpty}
      </p>
      <Show when={props.stored}>
        <button type="button" data-forget onClick={() => props.onForget()}>
          {props.text.provider.forget}
        </button>
      </Show>
    </>
  );
}

import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  active: string;
  onPick: (api: string) => void;
}

const APIS = ["ollama", "openai"];

export default function Apis(props: Props) {
  const label = (api: string) => {
    const names: Record<string, string> = {
      ollama: props.text.provider.apiOllama,
      openai: props.text.provider.apiOpenai,
    };
    return names[api] ?? api;
  };

  return (
    <fieldset>
      <legend>{props.text.provider.api}</legend>
      <For each={APIS}>
        {(api) => (
          <label>
            <input
              data-api={api}
              type="radio"
              name="provider-api"
              value={api}
              checked={props.active === api}
              onChange={() => props.onPick(api)}
            />
            {label(api)}
          </label>
        )}
      </For>
      <p data-api-hint>{props.text.provider.apiHint}</p>
    </fieldset>
  );
}

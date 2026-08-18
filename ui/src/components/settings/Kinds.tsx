import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";

interface Props {
  text: Dictionary;
  active: string;
  onPick: (kind: string) => void;
}

const KINDS = ["local", "remote", "harness"];

export default function Kinds(props: Props) {
  const label = (kind: string) => {
    const names: Record<string, string> = {
      local: props.text.provider.local,
      remote: props.text.provider.remote,
      harness: props.text.provider.harness,
    };
    return names[kind] ?? kind;
  };

  const privacy = () =>
    props.active === "local"
      ? props.text.provider.localPrivacy
      : props.text.provider.outsidePrivacy;

  return (
    <fieldset>
      <legend>{props.text.provider.kind}</legend>
      <For each={KINDS}>
        {(kind) => (
          <label>
            <input
              data-kind={kind}
              type="radio"
              name="provider-kind"
              value={kind}
              checked={props.active === kind}
              onChange={() => props.onPick(kind)}
            />
            {label(kind)}
          </label>
        )}
      </For>
      <p data-privacy>{privacy()}</p>
    </fieldset>
  );
}

import { For } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { AdviceView } from "../../ipc";

interface Props {
  text: Dictionary;
  advised: AdviceView[];
  chosen: string;
  onPick: (model: string) => void;
}

export default function Models(props: Props) {
  return (
    <section data-models>
      <h3>{props.text.provider.models}</h3>
      <p>{props.text.provider.modelsLead}</p>
      <For each={props.advised}>
        {(advice) => (
          <div data-advice>
            <span data-advice-model>{advice.model}</span>
            <span data-advice-size>
              {advice.gigabytes} {props.text.provider.gigabytes}
            </span>
            <span data-advice-fit>{fit(advice, props.text)}</span>
            <code data-advice-command>{advice.command}</code>
            <button
              type="button"
              data-advice-pick
              disabled={advice.model === props.chosen}
              onClick={() => props.onPick(advice.model)}
            >
              {props.text.provider.pick}
            </button>
          </div>
        )}
      </For>
    </section>
  );
}

function fit(advice: AdviceView, text: Dictionary): string {
  if (advice.installed) return text.provider.modelInstalled;
  return advice.heavy ? text.provider.modelHeavy : text.provider.modelFits;
}

import { For, Show } from "solid-js";

import type { Hints } from "../../i18n/hints/shape";
import type { Dictionary } from "../../i18n/ru";
import HintLines from "../HintLines";

interface Props {
  text: Dictionary;
  hints: Hints;
  preset: string;
}

export default function ArgsHint(props: Props) {
  const advised = () => props.hints.presets[props.preset] ?? [];

  return (
    <>
      <HintLines lines={props.hints.args} />
      <strong data-hint-advice>{props.hints.advice}</strong>
      <Show
        when={advised().length > 0}
        fallback={<span data-hint-own>{props.hints.own}</span>}
      >
        <span data-hint-advised role="list">
          <For each={advised()}>
            {(one) => (
              <span data-hint-arg role="listitem">
                <For each={one.lines}>
                  {(line) => (
                    <code>{line === "" ? props.text.provider.argsEmpty : line}</code>
                  )}
                </For>
                <span data-hint-why>{one.why}</span>
              </span>
            )}
          </For>
        </span>
      </Show>
    </>
  );
}

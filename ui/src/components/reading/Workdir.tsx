import { Show, createSignal } from "solid-js";

import type { Desk } from "./desk";
import type { Dictionary } from "../../i18n/ru";

interface Props {
  desk: Desk;
  text: Dictionary;
}

export default function Workdir(props: Props) {
  const [busy, setBusy] = createSignal(false);

  const choose = async () => {
    if (busy()) return;
    setBusy(true);
    await props.desk.choose();
    setBusy(false);
  };

  return (
    <p data-workdir>
      <Show when={props.desk.workdir()} fallback={<span data-unset>{props.text.stage.unset}</span>}>
        {(path) => (
          <span>
            {props.text.stage.workdir}: <code data-path>{path()}</code>
          </span>
        )}
      </Show>
      <button type="button" data-choose onClick={() => void choose()} aria-disabled={busy()}>
        {props.desk.workdir() === null ? props.text.stage.choose : props.text.stage.change}
      </button>
    </p>
  );
}

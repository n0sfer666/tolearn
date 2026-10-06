import { Show } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { DriftView } from "../../ipc";

interface Props {
  text: Dictionary;
  found: DriftView;
  onUpdate: () => void;
  onKeep: () => void;
}

export default function Drift(props: Props) {
  const reordered = () => props.found.removed.length === 0 && props.found.added.length === 0;

  return (
    <div data-drift>
      <p data-drift-title>{props.text.provider.driftTitle}</p>
      <Show when={props.found.removed.length > 0}>
        <p data-drift-removed>
          {props.text.provider.driftRemoved} {listed(props.found.removed)}
        </p>
      </Show>
      <Show when={props.found.added.length > 0}>
        <p data-drift-added>
          {props.text.provider.driftAdded} {listed(props.found.added)}
        </p>
      </Show>
      <Show when={reordered()}>
        <p data-drift-order>{props.text.provider.driftOrder}</p>
      </Show>
      <button type="button" data-drift-update onClick={() => props.onUpdate()}>
        {props.text.provider.driftUpdate}
      </button>
      <button type="button" data-drift-keep onClick={() => props.onKeep()}>
        {props.text.provider.driftKeep}
      </button>
    </div>
  );
}

function listed(args: string[]): string {
  return args.map((arg) => (arg === "" ? '""' : arg)).join(", ");
}

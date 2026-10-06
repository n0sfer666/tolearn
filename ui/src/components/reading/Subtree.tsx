import { Show } from "solid-js";

import type { SummaryView } from "../../ipc";
import { counted, summary } from "../../lib/summary";

interface Props {
  template: string;
  view: SummaryView | null;
}

export default function Subtree(props: Props) {
  return (
    <Show when={counted(props.view)}>
      {(view) => <span data-subtree>{summary(props.template, view())}</span>}
    </Show>
  );
}

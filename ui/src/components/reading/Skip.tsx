import { createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { StageIn } from "../../ipc";
import type { Transport } from "../../lib/ipc";
import { go as leave, nextHref } from "../../lib/links";
import { toast } from "../../lib/toast";

interface Props {
  text: Dictionary;
  locale: string;
  call: Transport;
  at: StageIn;
  go?: (href: string) => void;
}

export default function Skip(props: Props) {
  const href = () => nextHref(props.locale, props.at);
  const [busy, setBusy] = createSignal(false);

  const skip = async (event: MouseEvent) => {
    event.preventDefault();
    if (busy()) return;
    setBusy(true);
    try {
      await props.call("skip", props.at);
    } catch {
      setBusy(false);
      toast("error", props.text.stage.unskipped);
      return;
    }
    (props.go ?? leave)(href());
  };

  return (
    <a href={href()} data-next data-action aria-busy={busy()} onClick={(event) => void skip(event)}>
      {props.text.generate.skip}
    </a>
  );
}

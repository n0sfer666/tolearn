import { createSignal } from "solid-js";
import type { JSX } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { Transport } from "../../lib/ipc";
import { toast } from "../../lib/toast";
import { told } from "../../lib/told";

interface Props {
  text: Dictionary;
  program: string;
  node: string;
  call: Transport;
  pick: () => Promise<string | null>;
}

export default function Export(props: Props): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  const refusals = () =>
    new Map([
      ["export.occupied", props.text.program.occupied],
      ["export.inside-library", props.text.program.insideLibrary],
      ["export.folder", props.text.program.badFolder],
      ["export.unwritable", props.text.program.unwritable],
      ["export.asset", props.text.program.assetFailed],
    ]);

  const run = async () => {
    if (busy()) return;
    setBusy(true);
    try {
      const folder = await props.pick();
      if (folder === null) return;
      const out = await props.call("export", { program: props.program, node: props.node, folder });
      toast("ok", `${props.text.program.exported} ${out.path}`);
    } catch (failure) {
      toast("error", told(failure, refusals()) || props.text.program.exportFailed);
    } finally {
      setBusy(false);
    }
  };

  return (
    <button type="button" data-export data-action onClick={() => void run()} aria-disabled={busy()}>
      {props.text.program.export}
    </button>
  );
}

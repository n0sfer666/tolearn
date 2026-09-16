import { Show, createSignal } from "solid-js";
import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";

import type { Locale } from "../../i18n";
import { localized } from "../../i18n";
import type { Dictionary } from "../../i18n/ru";
import { grab } from "../../lib/grab";
import type { Transport } from "../../lib/ipc";
import { pressed } from "../../lib/shortcuts";
import { coded, explain, toast } from "../../lib/toast";
import { told } from "../../lib/told";

interface Props {
  text: Dictionary;
  locale: Locale;
  program: string;
  call: Transport;
  go: (href: string) => void;
  back: () => void;
  titled: () => string;
}

const LIBRARY = "/";
const HEADING = "delete-ask";

export default function Delete(props: Props): JSX.Element {
  const [asking, setAsking] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [name, setName] = createSignal("");
  let layer: HTMLDivElement | undefined;

  const refusals = () =>
    new Map([
      ["delete.subprogram", props.text.program.deleteSubprogram],
      ["delete.busy", props.text.program.deleteBusy],
      ["delete.left", props.text.program.deleteLeft],
      ["delete.cache", props.text.program.deleteCache],
      ["delete.broken", props.text.program.deleteBroken],
      ["library.absent", props.text.program.deleteGone],
    ]);

  const refused = (failure: unknown): string => {
    const known = told(failure, refusals()) || props.text.program.deleteFailed;
    return coded(failure) === "delete.left" ? `${known} ${explain(failure)}` : known;
  };

  const stops = (): HTMLElement[] => [...(layer?.querySelectorAll<HTMLElement>("button") ?? [])];

  const asked = async (): Promise<string> => {
    const out = await props.call("node", { program: props.program, node: "" });
    return out.trail[0]?.title ?? out.title;
  };

  const ask = () => {
    setName(props.titled());
    setAsking(true);
    if (name() !== "") return;
    void asked().then(setName, () => setName(""));
  };

  const close = () => {
    setAsking(false);
    props.back();
  };

  const run = async () => {
    if (busy()) return;
    setBusy(true);
    try {
      const out = await props.call("delete_program", { program: props.program });
      close();
      props.go(`${localized(LIBRARY, props.locale)}?${new URLSearchParams({ deleted: out.title })}`);
    } catch (failure) {
      close();
      toast("error", refused(failure));
    } finally {
      setBusy(false);
    }
  };

  const keyed = (event: KeyboardEvent) => {
    event.stopPropagation();
    if (pressed(event, "close")) {
      event.preventDefault();
      close();
      return;
    }
    if (!pressed(event, "cycle")) return;
    const row = stops();
    if (row.length === 0) return;
    const at = row.findIndex((stop) => stop === document.activeElement);
    event.preventDefault();
    row[(at + (event.shiftKey ? -1 : 1) + row.length) % row.length]?.focus();
  };

  const held = (event: FocusEvent) => {
    const next = event.relatedTarget;
    if (next instanceof Node && layer?.contains(next) === true) return;
    queueMicrotask(() => stops()[0]?.focus());
  };

  return (
    <>
      <button type="button" data-delete data-action onClick={ask}>
        {props.text.program.delete}
      </button>
      <Show when={asking()}>
        <Portal>
          <div data-confirm-back>
            <div
              data-confirm
              role="dialog"
              aria-modal="true"
              aria-labelledby={HEADING}
              ref={layer}
              on:keydown={keyed}
              onFocusOut={held}
            >
              <h2 id={HEADING}>{props.text.program.deleteAsk.replace("{name}", name())}</h2>
              <p>{props.text.program.deleteGoes}</p>
              <p>{props.text.program.deleteRestore}</p>
              <div data-confirm-answers>
                <button type="button" data-confirm-no ref={grab} onClick={close}>
                  {props.text.program.deleteNo}
                </button>
                <button
                  type="button"
                  data-confirm-yes
                  aria-disabled={busy()}
                  onClick={() => void run()}
                >
                  {props.text.program.deleteYes}
                </button>
              </div>
            </div>
          </div>
        </Portal>
      </Show>
    </>
  );
}

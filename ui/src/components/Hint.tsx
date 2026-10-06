import { type JSX, Show, createSignal } from "solid-js";
import { Portal } from "solid-js/web";

import { grab } from "../lib/grab";
import { layered } from "../lib/layered";

interface Props {
  label: string;
  shut: string;
  children: JSX.Element;
}

export default function Hint(props: Props) {
  const [open, setOpen] = createSignal(false);
  let opener: HTMLButtonElement | undefined;
  let box: HTMLDivElement | undefined;

  const close = () => {
    setOpen(false);
    opener?.focus();
  };

  const layer = layered(() => box, close, "button, [data-hint-body]");

  return (
    <>
      <button
        type="button"
        data-hint-open
        ref={opener}
        aria-label={props.label}
        aria-expanded={open()}
        onClick={() => (open() ? close() : setOpen(true))}
      >
        ?
      </button>
      <Show when={open()}>
        <Portal>
          <div data-hint-back onClick={layer.aside}>
            <div
              data-hint
              role="dialog"
              aria-modal="true"
              aria-label={props.label}
              ref={box}
              on:keydown={layer.keyed}
              onFocusOut={layer.held}
            >
              <button
                type="button"
                data-hint-close
                ref={grab}
                aria-label={props.shut}
                onClick={close}
              >
                ×
              </button>
              <div data-hint-body tabindex="0">
                {props.children}
              </div>
            </div>
          </div>
        </Portal>
      </Show>
    </>
  );
}

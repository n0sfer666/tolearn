import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";

import type { Dictionary } from "../../i18n/ru";
import { grab } from "../../lib/grab";
import { pressed } from "../../lib/shortcuts";

interface Props {
  src: string;
  alt: string;
  text: Dictionary;
  close: () => void;
}

export default function Zoom(props: Props): JSX.Element {
  let layer: HTMLDivElement | undefined;

  const stops = (): HTMLElement[] => [
    ...(layer?.querySelectorAll<HTMLElement>("button, [data-zoom-scroll]") ?? []),
  ];

  const keyed = (event: KeyboardEvent) => {
    event.stopPropagation();
    if (pressed(event, "close")) {
      event.preventDefault();
      props.close();
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

  const aside = (event: MouseEvent) => {
    if (event.target !== event.currentTarget) return;
    props.close();
  };

  return (
    <Portal>
      <div data-zoom-back onClick={aside}>
        <div
          data-zoom
          role="dialog"
          aria-modal="true"
          aria-label={props.alt}
          ref={layer}
          on:keydown={keyed}
          onFocusOut={held}
        >
          <button type="button" data-zoom-close ref={grab} onClick={() => props.close()}>
            {props.text.stage.shut}
          </button>
          <div data-zoom-scroll tabindex="0">
            <img src={props.src} alt={props.alt} />
          </div>
        </div>
      </div>
    </Portal>
  );
}

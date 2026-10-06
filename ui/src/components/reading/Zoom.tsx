import type { JSX } from "solid-js";
import { Portal } from "solid-js/web";

import type { Dictionary } from "../../i18n/ru";
import { grab } from "../../lib/grab";
import { layered } from "../../lib/layered";

interface Props {
  src: string;
  alt: string;
  text: Dictionary;
  close: () => void;
}

export default function Zoom(props: Props): JSX.Element {
  let box: HTMLDivElement | undefined;

  const layer = layered(() => box, () => props.close(), "button, [data-zoom-scroll]");

  return (
    <Portal>
      <div data-zoom-back onClick={layer.aside}>
        <div
          data-zoom
          role="dialog"
          aria-modal="true"
          aria-label={props.alt}
          ref={box}
          on:keydown={layer.keyed}
          onFocusOut={layer.held}
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

import { Show, createSignal } from "solid-js";

import Zoom from "./Zoom";
import type { Dictionary } from "../../i18n/ru";
import type { BlockView } from "../../ipc";

interface Props {
  block: BlockView;
  text: Dictionary;
}

export default function Picture(props: Props) {
  const [zoomed, setZoomed] = createSignal(false);
  const alt = () => (props.block.kind === "diagram" ? props.text.stage.diagram : props.block.text);
  let opener: HTMLButtonElement | undefined;

  const close = () => {
    setZoomed(false);
    opener?.focus();
  };

  return (
    <figure id={props.block.id} data-block={props.block.kind} tabindex="0">
      <Show when={props.block.src}>
        {(src) => (
          <>
            <button
              type="button"
              data-zoom-open
              ref={opener}
              aria-label={`${props.text.stage.expand}: ${alt()}`}
              onClick={() => setZoomed(true)}
            >
              <img src={src()} alt={alt()} />
            </button>
            <Show when={zoomed()}>
              <Zoom src={src()} alt={alt()} text={props.text} close={close} />
            </Show>
          </>
        )}
      </Show>
      <Show when={props.block.license}>
        {(license) => (
          <figcaption>
            <span data-license>
              {props.text.stage.license}: {license()}
            </span>
            <Show when={props.block.attribution}>
              {(author) => (
                <span data-attribution>
                  {props.text.stage.attribution}: {author()}
                </span>
              )}
            </Show>
            <Show when={props.block.source}>
              {(source) => (
                <span data-source>
                  {props.text.stage.source}: {source()}
                </span>
              )}
            </Show>
          </figcaption>
        )}
      </Show>
    </figure>
  );
}

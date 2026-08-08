import { type JSX, Show, createSignal, onCleanup } from "solid-js";

interface Props {
  label: string;
  seconds?: number;
  children: JSX.Element;
}

const PINNED_SECONDS = 10;

export default function Hint(props: Props) {
  const [over, setOver] = createSignal(false);
  const [pinned, setPinned] = createSignal(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const forget = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
  };

  const unpin = () => {
    forget();
    setPinned(false);
  };

  const pin = () => {
    forget();
    setPinned(true);
    timer = setTimeout(unpin, (props.seconds ?? PINNED_SECONDS) * 1000);
  };

  onCleanup(forget);

  const shown = () => over() || pinned();

  return (
    <span data-hint>
      <button
        type="button"
        data-hint-open
        aria-label={props.label}
        aria-expanded={shown()}
        onMouseEnter={() => setOver(true)}
        onMouseLeave={() => setOver(false)}
        onClick={() => (pinned() ? unpin() : pin())}
      >
        ?
      </button>
      <Show when={shown()}>
        <span data-hint-body role="tooltip">
          {props.children}
        </span>
      </Show>
    </span>
  );
}

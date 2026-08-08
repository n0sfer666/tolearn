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
  let opener: HTMLButtonElement | undefined;
  let body: HTMLElement | undefined;

  const forget = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
  };

  const inside = (target: EventTarget | null) => {
    if (!(target instanceof Node)) return false;
    return opener?.contains(target) === true || body?.contains(target) === true;
  };

  const aside = (event: Event) => {
    if (inside(event.target)) return;
    unpin();
  };

  const escaped = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    unpin();
  };

  const unpin = () => {
    forget();
    document.removeEventListener("pointerdown", aside);
    document.removeEventListener("keydown", escaped, true);
    setPinned(false);
  };

  const pin = () => {
    forget();
    setPinned(true);
    document.addEventListener("pointerdown", aside);
    document.addEventListener("keydown", escaped, true);
    timer = setTimeout(unpin, (props.seconds ?? PINNED_SECONDS) * 1000);
  };

  onCleanup(unpin);

  const shown = () => over() || pinned();

  return (
    <>
      <button
        type="button"
        data-hint-open
        ref={opener}
        aria-label={props.label}
        aria-expanded={shown()}
        onMouseEnter={() => setOver(true)}
        onMouseLeave={() => setOver(false)}
        onClick={() => (pinned() ? unpin() : pin())}
      >
        ?
      </button>
      <Show when={shown()}>
        <span data-hint-body role="tooltip" ref={body}>
          {props.children}
        </span>
      </Show>
    </>
  );
}

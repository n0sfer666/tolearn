import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { type Offer, SUMMONED, narrowed, offered } from "../lib/actions";
import { grab } from "../lib/grab";

interface ActionsText {
  title: string;
  query: string;
  none: string;
}

interface Props {
  text: ActionsText;
}

const LIST = "actions-list";

export default function Actions(props: Props) {
  const [open, setOpen] = createSignal(false);
  const [offers, setOffers] = createSignal<readonly Offer[]>([]);
  const [query, setQuery] = createSignal("");
  const [chosen, setChosen] = createSignal(0);
  const shown = createMemo(() => narrowed(offers(), query()));
  let before: Element | null = null;
  let layer: HTMLDivElement | undefined;

  const inside = (target: EventTarget | null) => target instanceof Node && layer?.contains(target) === true;

  const escaped = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    close(true);
  };

  const aside = (event: Event) => {
    if (!inside(event.target)) close(false);
  };

  const close = (back: boolean) => {
    if (!open()) return;
    document.removeEventListener("keydown", escaped, true);
    document.removeEventListener("pointerdown", aside);
    setOpen(false);
    if (back && before instanceof HTMLElement) before.focus();
    before = null;
  };

  const show = () => {
    before = document.activeElement;
    setOffers(offered(document));
    setQuery("");
    setChosen(0);
    setOpen(true);
    document.addEventListener("keydown", escaped, true);
    document.addEventListener("pointerdown", aside);
  };

  const toggle = () => (open() ? close(true) : show());

  const run = (offer: Offer | undefined) => {
    if (offer === undefined) return;
    close(true);
    offer.element.click();
  };

  const step = (by: number) => {
    const count = shown().length;
    if (count > 0) setChosen((at) => (at + by + count) % count);
  };

  const keyed = (event: KeyboardEvent) => {
    if (event.isComposing) return;
    if (event.key === "ArrowDown") step(1);
    else if (event.key === "ArrowUp") step(-1);
    else if (event.key === "Enter") run(shown()[chosen()]);
    else return;
    event.preventDefault();
  };

  const typed = (value: string) => {
    setQuery(value);
    setChosen(0);
  };

  onMount(() => {
    window.addEventListener(SUMMONED, toggle);
    onCleanup(() => {
      window.removeEventListener(SUMMONED, toggle);
      close(false);
    });
  });

  return (
    <Show when={open()}>
      <div
        data-actions
        role="dialog"
        aria-label={props.text.title}
        ref={layer}
        onFocusOut={(event) => {
          if (!inside(event.relatedTarget)) close(false);
        }}
      >
        <input
          type="text"
          data-actions-query
          ref={grab}
          role="combobox"
          aria-expanded="true"
          aria-controls={LIST}
          aria-activedescendant={shown().length > 0 ? option(chosen()) : undefined}
          aria-label={props.text.query}
          placeholder={props.text.query}
          value={query()}
          onInput={(event) => typed(event.currentTarget.value)}
          onKeyDown={keyed}
        />
        <ul id={LIST} role="listbox" aria-label={props.text.title} onMouseDown={(event) => event.preventDefault()}>
          <For each={shown()}>
            {(offer, index) => (
              <li id={option(index())} role="option" aria-selected={index() === chosen()} onClick={() => run(offer)}>
                {offer.label}
              </li>
            )}
          </For>
        </ul>
        <Show when={shown().length === 0}>
          <p data-actions-none>{props.text.none}</p>
        </Show>
      </div>
    </Show>
  );
}

function option(index: number): string {
  return `action-${index}`;
}

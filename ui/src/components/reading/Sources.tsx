import { For, Show } from "solid-js";
import type { JSX } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { SourcesView } from "../../ipc";
import { day } from "../../lib/day";

interface Props {
  text: Dictionary["program"];
  locale: string;
  view: SourcesView;
}

export default function Sources(props: Props): JSX.Element {
  const listed = () => props.view.books.length + props.view.pages.length > 0;

  return (
    <Show when={listed()}>
      <section data-sources>
        <h3>{props.text.sources}</h3>
        <ul>
          <For each={props.view.books}>
            {(book) => (
              <li data-book>
                {book.authors.join(", ")}. <cite>{book.title}</cite>
                <Show when={book.chapter !== ""}>
                  , {props.text.chapter} {book.chapter}
                </Show>
              </li>
            )}
          </For>
          <For each={props.view.pages}>
            {(page) => (
              <li data-page>
                <a href={page.url}>{page.title}</a>{" "}
                <span data-checked>
                  {props.text.checked} <time datetime={page.checked_at}>{day(page.checked_at, props.locale)}</time>
                </span>
              </li>
            )}
          </For>
        </ul>
      </section>
    </Show>
  );
}

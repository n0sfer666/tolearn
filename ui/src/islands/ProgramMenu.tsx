import { Show } from "solid-js";
import type { JSX } from "solid-js";

import Delete from "../components/reading/Delete";
import Export from "../components/reading/Export";
import type { Locale } from "../i18n";
import type { Dictionary } from "../i18n/ru";
import { pickFolder, quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { go } from "../lib/links";
import { named } from "../lib/name";
import { query } from "../lib/query";

interface Props {
  text: Dictionary;
  locale: Locale;
  program?: string;
  node?: string;
  call?: Transport;
  pick?: () => Promise<string | null>;
  go?: (href: string) => void;
  titled?: () => string;
}

export default function ProgramMenu(props: Props): JSX.Element {
  let menu: HTMLDetailsElement | undefined;
  let trigger: HTMLElement | undefined;
  const close = () => {
    if (menu) menu.open = false;
  };
  const program = () => props.program ?? query("program");
  const node = () => props.node ?? query("node");
  const call = () => props.call ?? quiet;

  return (
    <details data-menu ref={menu}>
      <summary ref={trigger} aria-label={props.text.program.menu}>
        ⋯
      </summary>
      <div data-menu-items onClick={close}>
        <Export
          text={props.text}
          program={program()}
          node={node()}
          call={call()}
          pick={props.pick ?? pickFolder}
        />
        <Show when={node() === ""}>
          <Delete
            text={props.text}
            locale={props.locale}
            program={program()}
            call={call()}
            go={props.go ?? go}
            back={() => trigger?.focus()}
            titled={props.titled ?? (() => named("program", program()))}
          />
        </Show>
      </div>
    </details>
  );
}

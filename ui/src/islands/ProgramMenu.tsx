import type { JSX } from "solid-js";

import Export from "../components/reading/Export";
import type { Dictionary } from "../i18n/ru";
import { pickFolder, quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { query } from "../lib/query";

interface Props {
  text: Dictionary;
  program?: string;
  node?: string;
  call?: Transport;
  pick?: () => Promise<string | null>;
}

export default function ProgramMenu(props: Props): JSX.Element {
  let menu: HTMLDetailsElement | undefined;
  const close = () => {
    if (menu) menu.open = false;
  };

  return (
    <details data-menu ref={menu}>
      <summary aria-label={props.text.program.menu}>⋯</summary>
      <div data-menu-items onClick={close}>
        <Export
          text={props.text}
          program={props.program ?? query("program")}
          node={props.node ?? query("node")}
          call={props.call ?? quiet}
          pick={props.pick ?? pickFolder}
        />
      </div>
    </details>
  );
}

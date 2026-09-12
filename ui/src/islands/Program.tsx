import { Show, createSignal, onMount } from "solid-js";

import Empty from "../components/reading/Empty";
import Export from "../components/reading/Export";
import Rows from "../components/reading/Rows";
import Trail from "../components/reading/Trail";
import type { Dictionary } from "../i18n/ru";
import type { NodeOut } from "../ipc";
import { hours } from "../lib/hours";
import { pickFolder, quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { nodeHref, stageHref } from "../lib/links";
import { name } from "../lib/name";
import { query } from "../lib/query";
import { told } from "../lib/told";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  call?: Transport;
  pick?: () => Promise<string | null>;
}

export default function Program(props: Props) {
  const call = () => props.call ?? quiet;
  const program = () => props.program ?? query("program");
  const node = () => props.node ?? query("node");

  const [view, setView] = createSignal<NodeOut | null>(null);
  const [gone, setGone] = createSignal<string | null>(null);
  const missing = () =>
    new Map([
      ["library.absent", props.text.program.absent],
      ["node.absent", props.text.program.nodeAbsent],
    ]);

  const open = async () => {
    try {
      const out = await call()("node", { program: program(), node: node() });
      setView(out);
      name("program", out.program, out.trail[0]?.title ?? out.title);
    } catch (failure) {
      setGone(told(failure, missing()) || props.text.program.none);
    }
  };

  onMount(() => {
    if (program() === "") {
      setGone(props.text.program.none);
      return;
    }
    void open();
  });

  const crumbs = (out: NodeOut) =>
    out.trail.map((crumb) => ({
      href: nodeHref(props.locale, out.program, crumb.uuid),
      title: crumb.title,
    }));

  return (
    <Show
      when={view()}
      fallback={
        <Show when={gone()}>
          {(reason) => <Empty reason={reason()} locale={props.locale} label={props.text.nav.programs} />}
        </Show>
      }
    >
      {(out) => (
        <article data-node={out().uuid}>
          <Show when={out().trail.length > 0}>
            <Trail label={props.text.program.trail} crumbs={crumbs(out())} />
            <h2 data-node-title>{out().title}</h2>
          </Show>
          <p data-goal>
            <strong>{props.text.program.goal}:</strong> {out().goal}
          </p>
          <p data-level>
            <strong>{props.text.program.level}:</strong> {out().level}
          </p>
          <p data-node-hours>
            <strong>{props.text.program.span}:</strong> {hours(out().hours, props.text.program.hours)}
          </p>
          <Export
            text={props.text}
            program={out().program}
            node={out().uuid}
            call={call()}
            pick={props.pick ?? pickFolder}
          />

          <Show when={out().stages.length > 0}>
            <h2>{props.text.program.stages}</h2>
            <ol data-stages>
              <Rows
                kind="stage"
                rows={out().stages}
                href={(row) => stageHref(props.locale, out().program, out().uuid, row.id)}
                pending={props.text.program.pending}
                unit={props.text.program.hours}
              />
            </ol>
          </Show>

          <Show when={out().children.length > 0}>
            <h2>{props.text.program.children}</h2>
            <ul data-children>
              <Rows
                kind="child"
                rows={out().children}
                href={(row) => nodeHref(props.locale, out().program, row.id)}
                pending={props.text.program.pending}
                unit={props.text.program.hours}
              />
            </ul>
          </Show>
        </article>
      )}
    </Show>
  );
}

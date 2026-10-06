import { Show, createSignal, onMount } from "solid-js";

import Empty from "../components/reading/Empty";
import Here from "../components/reading/Here";
import Mark from "../components/reading/Mark";
import Rows from "../components/reading/Rows";
import Sources from "../components/reading/Sources";
import Subtree from "../components/reading/Subtree";
import Trail from "../components/reading/Trail";
import type { Dictionary } from "../i18n/ru";
import type { NodeOut, StageRowView } from "../ipc";
import { hours } from "../lib/hours";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { nodeHref, stageHref } from "../lib/links";
import { name } from "../lib/name";
import { query } from "../lib/query";
import { summary } from "../lib/summary";
import { told } from "../lib/told";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  call?: Transport;
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
  const next = (out: NodeOut) => out.stages.find((row) => row.ready && row.status !== "passed")?.id;
  const stage = (out: NodeOut, row: StageRowView) => stageHref(props.locale, out.program, out.uuid, row.id);

  return (
    <Show
      when={view()}
      fallback={
        <Show when={gone()}>
          {(reason) => <Empty reason={reason()} locale={props.locale} label={props.text.nav.library} />}
        </Show>
      }
    >
      {(out) => (
        <article data-node={out().uuid}>
          <Show when={out().trail.length > 0}>
            <Trail label={props.text.program.trail} crumbs={crumbs(out())} />
            <h2 data-node-title>{out().title}</h2>
          </Show>
          <div data-chips>
            <p data-node-hours>
              <strong>{props.text.program.span}:</strong> {hours(out().hours, props.text.program.hours)}
            </p>
            <Show when={out().summary.total > 0}>
              <p data-summary>
                <strong>{props.text.program.progress}:</strong> {summary(props.text.program.summary, out().summary)}
              </p>
            </Show>
          </div>

          <Show when={out().stages.length > 0}>
            <h2>{props.text.program.stages}</h2>
            <ol data-stages data-route>
              <Rows
                kind="stage"
                rows={out().stages}
                href={(row) => stage(out(), row)}
                pending={props.text.program.pending}
                unit={props.text.program.hours}
                mark={(row) => (
                  <>
                    <Mark text={props.text.program} row={row} />
                    <Show when={row.id === next(out())}>
                      <Here here={props.text.program.here} resume={props.text.program.resume} href={stage(out(), row)} />
                    </Show>
                  </>
                )}
              />
            </ol>
          </Show>

          <Show when={out().children.length > 0}>
            <h2>{props.text.program.children}</h2>
            <ul data-children data-route>
              <Rows
                kind="child"
                rows={out().children}
                href={(row) => nodeHref(props.locale, out().program, row.id)}
                pending={props.text.program.pending}
                unit={props.text.program.hours}
                mark={(row) => <Subtree template={props.text.program.summary} view={row.summary} />}
              />
            </ul>
          </Show>

          <div data-facts>
            <div>
              <p data-goal>
                <strong>{props.text.program.goal}:</strong> {out().goal}
              </p>
              <p data-level>
                <strong>{props.text.program.level}:</strong> {out().level}
              </p>
            </div>
            <Sources text={props.text.program} locale={props.locale} view={out().sources} />
          </div>
        </article>
      )}
    </Show>
  );
}

import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import { GLYPHS, known, label } from "../components/status";
import type { Dictionary } from "../i18n/ru";
import type { GraphOut, NodeView } from "../ipc";
import { transport } from "../lib/ipc";
import type { Transport } from "../lib/ipc";

interface Props {
  text: Dictionary;
  locale: string;
  path?: string;
  today?: string;
  call?: Transport;
}

interface Layer {
  n: number;
  nodes: NodeView[];
}

export default function Graph(props: Props) {
  const call = () => props.call ?? transport;
  const today = () => props.today ?? new Date().toISOString().slice(0, 10);
  const path = () => props.path ?? new URLSearchParams(location.search).get("program") ?? "";

  const [taken, setTaken] = createSignal<GraphOut | null>(null);

  onMount(() => {
    void (async () => {
      setTaken(await call()("graph", { bundle: path(), today: today() }));
    })();
  });

  const nodes = () => taken()?.nodes ?? [];

  const layers = createMemo(() => {
    const kept: Layer[] = [];
    for (const node of nodes()) {
      const last = kept[kept.length - 1];
      if (last !== undefined && last.n === node.layer) {
        last.nodes.push(node);
        continue;
      }
      kept.push({ n: node.layer, nodes: [node] });
    }
    return kept;
  });

  const titles = createMemo(() => new Map(nodes().map((node) => [node.id, node.title])));
  const named = (id: string) => titles().get(id) ?? id;

  const href = (topic: string) =>
    `/${props.locale}/topic/?program=${encodeURIComponent(path())}&topic=${encodeURIComponent(topic)}`;

  const glyph = (status: string) => (known(status) ? GLYPHS[status] : "·");

  const link = (id: string) => (
    <li>
      <a href={href(id)}>{named(id)}</a>
    </li>
  );

  const node = (node: NodeView) => (
    <li data-node={node.id} data-status={node.status}>
      <a data-open href={href(node.id)}>
        <span data-glyph aria-hidden="true">
          {glyph(node.status)}
        </span>
        {node.title}
      </a>
      <span data-state>{label(props.text, node.status)}</span>
      <Show
        when={node.blocked_by.length > 0}
        fallback={<span data-free>{props.text.graph.free}</span>}
      >
        <div data-waiting>
          {props.text.graph.waiting}:
          <ul>
            <For each={node.blocked_by}>{link}</For>
          </ul>
        </div>
      </Show>
      <Show
        when={node.unlocks.length > 0}
        fallback={<span data-leaf>{props.text.graph.leaf}</span>}
      >
        <div data-unlocks>
          {props.text.graph.unlocks}:
          <ul>
            <For each={node.unlocks}>{link}</For>
          </ul>
        </div>
      </Show>
    </li>
  );

  return (
    <Show when={taken()}>
      <Show when={nodes().length > 0} fallback={<p data-empty>{props.text.graph.empty}</p>}>
        <ol data-graph>
          <For each={layers()}>
            {(layer) => (
              <li data-layer={layer.n}>
                <h3>
                  {props.text.graph.layer} {layer.n}
                </h3>
                <ul>
                  <For each={layer.nodes}>{node}</For>
                </ul>
              </li>
            )}
          </For>
        </ol>
      </Show>
    </Show>
  );
}

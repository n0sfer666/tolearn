import { For, Show, createSignal, onMount } from "solid-js";

import Block from "../components/reading/Block";
import Empty from "../components/reading/Empty";
import Practice from "../components/reading/Practice";
import Trail from "../components/reading/Trail";
import Rich from "../components/rich/Rich";
import type { Copier, Words } from "../components/rich/Snip";
import type { Dictionary } from "../i18n/ru";
import type { StageOut } from "../ipc";
import { reveal } from "../lib/anchor";
import { quiet } from "../lib/ipc";
import type { Transport } from "../lib/ipc";
import { nodeHref } from "../lib/links";
import { query } from "../lib/query";
import { told } from "../lib/told";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  stage?: string;
  call?: Transport;
  copy?: Copier;
}

export default function Stage(props: Props) {
  const call = () => props.call ?? quiet;
  const program = () => props.program ?? query("program");
  const node = () => props.node ?? query("node");
  const id = () => props.stage ?? query("stage");
  const words = (): Words => ({
    copied: props.text.stage.copied,
    manual: props.text.stage.manual,
    label: props.text.stage.snip,
  });
  const missing = () =>
    new Map([
      ["library.absent", props.text.program.absent],
      ["node.absent", props.text.program.nodeAbsent],
      ["stage.absent", props.text.stage.absent],
      ["library.foreign", props.text.stage.foreign],
    ]);

  const [view, setView] = createSignal<StageOut | null>(null);
  const [gone, setGone] = createSignal<string | null>(null);

  onMount(() => {
    if (program() === "" || id() === "") {
      setGone(props.text.stage.none);
      return;
    }
    void (async () => {
      try {
        setView(await call()("stage", { program: program(), node: node(), stage: id() }));
        reveal();
      } catch (failure) {
        setGone(told(failure, missing()) || props.text.stage.none);
      }
    })();
  });

  const up = (out: StageOut) => [
    { href: nodeHref(props.locale, out.program, out.node), title: out.node_title },
  ];

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
        <article data-stage-view>
          <Trail label={props.text.program.trail} crumbs={up(out())} />
          <h2 data-stage-title>{out().title}</h2>
          <div data-stage-body>
            <For each={out().blocks}>
              {(block) => <Block block={block} text={props.text} words={words()} copy={props.copy} />}
            </For>
          </div>
          <Practice practice={out().practice} text={props.text} words={words()} copy={props.copy} />
          <Show when={out().questions.length > 0}>
            <section data-questions>
              <h3>{props.text.stage.questions}</h3>
              <ol>
                <For each={out().questions}>
                  {(ask) => (
                    <li id={ask.id} data-question={ask.id}>
                      <Rich text={ask.text} words={words()} copy={props.copy} />
                    </li>
                  )}
                </For>
              </ol>
            </section>
          </Show>
        </article>
      )}
    </Show>
  );
}

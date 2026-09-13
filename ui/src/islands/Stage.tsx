import { For, Show, createSignal, onMount } from "solid-js";

import Regenerate from "../components/generate/Regenerate";
import Block from "../components/reading/Block";
import Empty from "../components/reading/Empty";
import Practice from "../components/reading/Practice";
import Question from "../components/reading/Question";
import { desk } from "../components/reading/desk";
import Trail from "../components/reading/Trail";
import type { Copier, Words } from "../components/rich/Snip";
import type { Dictionary } from "../i18n/ru";
import type { StageOut } from "../ipc";
import { reveal } from "../lib/anchor";
import { pickFolder, quiet, steps } from "../lib/ipc";
import type { Listen, Transport } from "../lib/ipc";
import { nextHref, nodeHref } from "../lib/links";
import { query } from "../lib/query";
import { toast } from "../lib/toast";
import { told } from "../lib/told";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  stage?: string;
  call?: Transport;
  steps?: Listen;
  copy?: Copier;
  pick?: () => Promise<string | null>;
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

  const fetched = () => call()("stage", { program: program(), node: node(), stage: id() });

  const load = async () => {
    try {
      setView(await fetched());
      reveal();
    } catch (failure) {
      setGone(told(failure, missing()) || props.text.stage.none);
    }
  };

  const reload = async () => {
    try {
      setView(await fetched());
    } catch {
      toast("error", props.text.generate.unread);
    }
  };

  onMount(() => {
    if (program() === "" || id() === "") {
      setGone(props.text.stage.none);
      return;
    }
    void load();
  });

  const bench = (out: () => StageOut) =>
    desk({
      call: call(),
      pick: props.pick ?? pickFolder,
      at: () => ({ program: out().program, node: out().node, stage: out().id }),
      text: props.text,
      ticks: () => out().ticks,
      workdir: () => out().workdir,
      ticked: (ticks) => setView({ ...out(), ticks }),
      chosen: (workdir) => setView({ ...out(), workdir }),
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
          <Practice practice={out().practice} desk={bench(out)} text={props.text} words={words()} copy={props.copy} />
          <Show when={out().questions.length > 0}>
            <section data-questions>
              <h3>{props.text.stage.questions}</h3>
              <ol>
                <For each={out().questions}>
                  {(ask) => <Question ask={ask} text={props.text} words={words()} copy={props.copy} />}
                </For>
              </ol>
            </section>
          </Show>
          <footer data-stage-end>
            <nav data-tools aria-label={props.text.generate.tools}>
              <a href={nextHref(props.locale, out().program, out().node, out().id)} data-next>
                {props.text.generate.skip}
              </a>
            </nav>
            <Regenerate
              text={props.text}
              locale={props.locale}
              call={call()}
              listen={props.steps ?? steps}
              at={{ program: out().program, node: out().node, stage: out().id }}
              done={() => void reload()}
            />
          </footer>
        </article>
      )}
    </Show>
  );
}

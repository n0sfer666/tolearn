import { Show, createSignal, onMount } from "solid-js";

import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import Variants from "../components/generate/Variants";
import Empty from "../components/reading/Empty";
import type { Dictionary } from "../i18n/ru";
import type { VariantView } from "../ipc";
import { generation } from "../lib/generation";
import { quiet, steps } from "../lib/ipc";
import type { Listen, Transport } from "../lib/ipc";
import { BUILDING, FORKING } from "../lib/jobs";
import { go, stageHref } from "../lib/links";
import { query } from "../lib/query";
import { regain } from "../lib/regain";

interface Props {
  text: Dictionary;
  locale: string;
  program?: string;
  node?: string;
  stage?: string;
  call?: Transport;
  steps?: Listen;
  go?: (href: string) => void;
}

export default function Next(props: Props) {
  const call = () => props.call ?? quiet;
  const work = generation(props.text, call, props.steps ?? steps);
  const at = () => ({
    program: props.program ?? query("program"),
    node: props.node ?? query("node"),
    stage: props.stage ?? query("stage"),
  });

  const [variants, setVariants] = createSignal<VariantView[]>([]);
  const [opened, setOpened] = createSignal(false);
  const [missing, setMissing] = createSignal(false);

  const open = async () => {
    setOpened(true);
    const done = await work.run(() => call()("fork", at()), FORKING);
    if (done !== null) setVariants(done.variants);
  };

  const choose = async (choice: number) => {
    const here = at();
    const done = await work.run(() => call()("take_next", { ...here, choice }), BUILDING);
    if (done !== null) (props.go ?? go)(stageHref(props.locale, here.program, done.node, done.stage));
  };

  onMount(() => {
    const here = at();
    if (here.program !== "" && here.stage !== "") {
      void open();
      return;
    }
    setMissing(true);
  });

  const idle = () => opened() && !work.running();

  return (
    <Show
      when={!missing()}
      fallback={<Empty reason={props.text.generate.none} locale={props.locale} label={props.text.nav.programs} />}
    >
      <h2>{props.text.generate.variants}</h2>
      <p>{props.text.generate.variantsLead}</p>
      <div data-fork>
        <Show when={work.running()}>
          <Progress text={props.text} work={work} />
        </Show>
        <Show when={idle() && variants().length > 0}>
          <Variants
            text={props.text}
            variants={variants()}
            focus={regain(work.calm)}
            choose={(choice) => void choose(choice)}
          />
        </Show>
        <Show when={idle() && variants().length === 0}>
          <button type="button" data-retry ref={regain(() => work.ended() === "halted")} onClick={() => void open()}>
            {props.text.generate.retry}
          </button>
        </Show>
        <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
      </div>
    </Show>
  );
}

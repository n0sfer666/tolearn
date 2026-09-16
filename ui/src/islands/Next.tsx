import { Show, createMemo, createSignal, onMount } from "solid-js";

import Ahead from "../components/generate/Ahead";
import Outcome from "../components/generate/Outcome";
import Progress from "../components/generate/Progress";
import Refusal from "../components/generate/Refusal";
import Variants from "../components/generate/Variants";
import Empty from "../components/reading/Empty";
import type { Dictionary } from "../i18n/ru";
import type { NodeOut, StageOut, VariantView } from "../ipc";
import { generation } from "../lib/generation";
import { quiet, steps } from "../lib/ipc";
import type { Listen, Transport } from "../lib/ipc";
import { BUILDING, FORKING } from "../lib/jobs";
import { go, stageHref } from "../lib/links";
import { query } from "../lib/query";
import { ranked } from "../lib/ranked";
import { regain } from "../lib/regain";
import { slot } from "../lib/slot";

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
  const [map, setMap] = createSignal<NodeOut | null>(null);
  const [done, setDone] = createSignal<StageOut | null>(null);
  const [hovered, setHovered] = createSignal<number | null>(null);
  const [focused, setFocused] = createSignal(0);

  const picks = createMemo(() => ranked(variants()));
  const taken = createMemo(() => slot(map(), at().stage));
  const picked = () => hovered() ?? focused();
  const rows = () => map()?.stages.length ?? 0;
  const place = () => {
    const where = taken();
    if (where === null) return null;
    if (where >= rows()) return props.text.generate.placeOnward;
    return props.text.generate.place.replace("{n}", String(where + 1)).replace("{m}", String(rows()));
  };
  const row = () => map()?.stages.find((stage) => stage.id === at().stage) ?? null;

  const read = async () => {
    const here = at();
    const [node, stage] = await Promise.allSettled([
      call()("node", { program: here.program, node: here.node }),
      call()("stage", here),
    ]);
    setMap(node.status === "fulfilled" ? node.value : null);
    setDone(stage.status === "fulfilled" ? stage.value : null);
  };

  const open = async () => {
    setOpened(true);
    setHovered(null);
    setFocused(0);
    const found = await work.run(() => call()("fork", at()), FORKING);
    if (found !== null) setVariants(found.variants);
  };

  const choose = async (choice: number) => {
    const here = at();
    const built = await work.run(() => call()("take_next", { ...here, choice }), BUILDING);
    if (built !== null) (props.go ?? go)(stageHref(props.locale, here.program, built.node, built.stage));
  };

  onMount(() => {
    const here = at();
    if (here.program !== "" && here.stage !== "") {
      void read();
      void open();
      return;
    }
    setMissing(true);
  });

  const idle = () => opened() && !work.running();

  return (
    <Show
      when={!missing()}
      fallback={<Empty reason={props.text.generate.none} locale={props.locale} label={props.text.nav.library} />}
    >
      <div data-fork data-studio>
        <div data-studio-ask>
          <h2>{props.text.generate.variants}</h2>
          <p>{props.text.generate.variantsLead}</p>
          <Show when={work.running()}>
            <Progress text={props.text} work={work} />
          </Show>
          <Show when={idle() && picks().length > 0}>
            <Variants
              text={props.text}
              picks={picks()}
              picked={picked()}
              place={place()}
              focus={regain(work.calm)}
              attend={setFocused}
              hover={setHovered}
              choose={(choice) => void choose(choice)}
            />
          </Show>
          <Show when={idle() && picks().length === 0}>
            <button type="button" data-retry ref={regain(() => work.ended() === "halted")} onClick={() => void open()}>
              {props.text.generate.retry}
            </button>
          </Show>
          <Refusal text={props.text} locale={props.locale} refused={work.refused()} />
        </div>
        <div data-studio-side>
          <Show when={done()}>{(view) => <Outcome text={props.text} view={view()} pass={row()?.pass ?? null} />}</Show>
        </div>
        <Show when={map() !== null && taken() !== null && picks().length > 0}>
          <Ahead
            text={props.text}
            stages={map()?.stages ?? []}
            slot={taken() ?? 0}
            title={picks()[picked()]?.variant.title ?? ""}
          />
        </Show>
      </div>
    </Show>
  );
}

import { For, Show, createSignal } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { GenerateStateOut } from "../../ipc";
import { crunching, tail } from "../../lib/crunch";

interface Props {
  text: Dictionary["generate"];
  live: GenerateStateOut | null;
  onGo: () => void;
  onStop: () => void;
  onClose: () => void;
}

export default function Going(props: Props) {
  const [leaving, setLeaving] = createSignal(false);

  const where = () => {
    const live = props.live;
    if (live === null || live.step === "skeleton") return props.text.skeleton;
    if (live.step === "confirm") return props.text.confirm;
    if (live.step === "missed") return props.text.missing;
    if (live.finished) return `${props.text.gathered}: ${live.done} / ${live.total}`;
    const at = Math.min(live.done + 1, live.total);
    return `${props.text.topic} ${at} / ${live.total}: ${live.current}`;
  };

  const cost = () => {
    const live = props.live;
    if (live === null) return "";
    if (live.tokens === null) return props.text.silent;
    return `${props.text.spent}: ${live.tokens} ${props.text.tokens}`;
  };

  const done = () => props.live?.finished === true;
  const beat = () => {
    const live = props.live;
    if (live === null || done()) return "";
    return crunching(live, props.text);
  };
  const shown = () => {
    const live = props.live;
    return live === null || done() ? [] : tail(live);
  };
  const asks = () => props.live?.waiting === true && !done();
  const refused = () => props.live?.refused ?? [];
  const missed = () => props.live?.missed ?? [];
  const stuck = () => missed().length > 0 && !done();

  return (
    <div data-generating>
      <p data-generate-step>{where()}</p>
      <Show when={(props.live?.attempt ?? 0) > 1}>
        <p data-generate-attempt>
          {props.text.attempt} {props.live?.attempt} / {props.live?.rounds}
        </p>
      </Show>
      <Show when={(props.live?.retry ?? 0) > 1 && !done()}>
        <p data-generate-retry>
          {props.text.reconnect} {props.live?.retry} / {props.live?.tries}
        </p>
      </Show>
      <Show when={beat() !== ""}>
        <p data-generate-beat>{beat()}</p>
      </Show>
      <p data-generate-cost>{cost()}</p>
      <Show when={shown().length > 0}>
        <pre data-generate-tail>{shown().join("\n")}</pre>
      </Show>
      <Show when={stuck()}>
        <p data-generate-gathered>
          {props.text.gathered}: {props.live?.done} / {props.live?.total}
        </p>
        <ul data-generate-missed>
          <For each={missed()}>{(why) => <li>{why}</li>}</For>
        </ul>
        <button type="button" data-generate-again onClick={props.onGo}>
          {props.text.again}
        </button>
      </Show>
      <Show when={asks()}>
        <p data-generate-estimate>
          {props.text.estimate}: {props.live?.total}
        </p>
        <button type="button" data-generate-go onClick={props.onGo}>
          {props.text.go}
        </button>
      </Show>
      <Show when={!done() && !leaving()}>
        <button type="button" data-generate-stop onClick={() => setLeaving(true)}>
          {props.text.abandon}
        </button>
      </Show>
      <Show when={!done() && leaving()}>
        <p data-generate-abandon-ask>{props.text.abandonAsk}</p>
        <button type="button" data-generate-abandon onClick={props.onStop}>
          {props.text.abandonYes}
        </button>
        <button type="button" data-generate-keep onClick={() => setLeaving(false)}>
          {props.text.keep}
        </button>
      </Show>
      <Show when={done()}>
        <p data-generate-outcome>
          {props.live?.cancelled === true ? props.text.cancelled : props.text.stopped}
        </p>
        <Show when={refused().length > 0}>
          <ul data-generate-refused>
            <For each={refused()}>{(why) => <li>{why}</li>}</For>
          </ul>
        </Show>
        <p data-generate-saved>{props.text.saved}</p>
        <button type="button" data-generate-close onClick={props.onClose}>
          {props.text.close}
        </button>
      </Show>
    </div>
  );
}

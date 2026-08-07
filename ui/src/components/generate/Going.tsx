import { For, Show } from "solid-js";

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
  const where = () => {
    const live = props.live;
    if (live === null || live.step === "skeleton") return props.text.skeleton;
    if (live.step === "confirm") return props.text.confirm;
    return `${props.text.topic} ${live.done + 1} / ${live.total}: ${live.current}`;
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
      <Show when={asks()}>
        <p data-generate-estimate>
          {props.text.estimate}: {props.live?.total}
        </p>
        <button type="button" data-generate-go onClick={props.onGo}>
          {props.text.go}
        </button>
      </Show>
      <Show
        when={done()}
        fallback={
          <button type="button" data-generate-stop onClick={props.onStop}>
            {props.text.cancel}
          </button>
        }
      >
        <p data-generate-outcome>
          {props.live?.cancelled === true ? props.text.cancelled : props.text.stopped}
        </p>
        <Show when={refused().length > 0}>
          <ul data-generate-refused>
            <For each={refused()}>{(why) => <li>{why}</li>}</For>
          </ul>
        </Show>
        <button type="button" data-generate-close onClick={props.onClose}>
          {props.text.close}
        </button>
      </Show>
    </div>
  );
}

import { createSignal, onCleanup, onMount } from "solid-js";
import type { Accessor } from "solid-js";

import type { GenerationOutcome, GenerationStateOut, GenerationWork, RefusalView } from "../ipc";
import type { Transport, Watch } from "./ipc";

export interface Ongoing {
  state: Accessor<GenerationStateOut | null>;
  known: Promise<GenerationStateOut>;
  ended: <TPicked>(work: GenerationWork, pick: (outcome: GenerationOutcome) => TPicked | null) => Promise<TPicked>;
  seen: () => void;
}

type Ending = { outcome: GenerationOutcome } | { failure: RefusalView };

interface Waiter {
  began: number;
  settle: (ending: Ending) => void;
}

const EMPTY: GenerationStateOut = { work: null, outcome: null };
const CANCELLED: RefusalView = { code: "generate.cancelled", message: "" };
const UNNAMED: RefusalView = { code: "generate.failed", message: "" };

function finished(state: GenerationStateOut, began: number): Ending | null {
  if (state.work?.began === began) return null;
  const outcome = state.outcome;
  if (outcome === null || outcome.work.began !== began) return { failure: CANCELLED };
  if (outcome.refusal !== null) return { failure: outcome.refusal };
  return { outcome };
}

export function ongoing(call: () => Transport, watch: Watch): Ongoing {
  const [state, setState] = createSignal<GenerationStateOut | null>(null);
  const waiters = new Set<Waiter>();
  let fresh = 0;
  let know: (state: GenerationStateOut) => void = () => undefined;
  const known = new Promise<GenerationStateOut>((resolve) => {
    know = resolve;
  });

  const take = (next: GenerationStateOut) => {
    setState(next);
    know(next);
    for (const waiter of [...waiters]) {
      const ending = finished(next, waiter.began);
      if (ending === null) continue;
      waiters.delete(waiter);
      waiter.settle(ending);
    }
  };

  const heard = (next: GenerationStateOut) => {
    fresh += 1;
    take(next);
  };

  const unless = (mine: number) => (next: GenerationStateOut) => {
    if (mine === fresh) take(next);
  };

  const read = () => {
    const fill = unless(fresh);
    void call()("generation_state", {}).then(fill, () => fill(EMPTY));
  };

  onMount(() => {
    onCleanup(watch(heard));
    read();
  });

  const ended = <TPicked>(work: GenerationWork, pick: (outcome: GenerationOutcome) => TPicked | null) =>
    new Promise<TPicked>((resolve, reject) => {
      const settle = (ending: Ending) => {
        const picked = "outcome" in ending ? pick(ending.outcome) : null;
        if (picked !== null) resolve(picked);
        else reject("failure" in ending ? ending.failure : UNNAMED);
      };
      const now = state();
      const ending = now === null ? null : finished(now, work.began);
      if (ending === null) waiters.add({ began: work.began, settle });
      else settle(ending);
    });

  const seen = () => {
    void call()("generation_seen", {}).then(unless(fresh), () => undefined);
  };

  return { state, known, ended, seen };
}

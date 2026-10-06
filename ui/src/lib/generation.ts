import { createSignal, onCleanup, onMount } from "solid-js";
import type { Accessor } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { CancelGenerationOut, GenerationStep, GenerationWork } from "../ipc";
import { acknowledge } from "./acknowledge";
import type { Listen, Transport } from "./ipc";
import type { Job } from "./jobs";
import { elapsed } from "./elapsed";
import { refusal } from "./refusal";
import type { Refused } from "./refusal";
import { stepped } from "./stepped";
import { coded, explain, toast } from "./toast";

type Outcome<TDone> = { ok: true; done: TDone } | { ok: false; failure: unknown };

export type Ending = "done" | "halted" | "refused";

export interface Generation {
  step: Accessor<GenerationStep | null>;
  said: Accessor<string>;
  spent: Accessor<number>;
  began: Accessor<number | null>;
  running: Accessor<boolean>;
  cancelling: Accessor<boolean>;
  ended: Accessor<Ending | null>;
  calm: Accessor<boolean>;
  refused: Accessor<Refused | null>;
  refuse: (reason: string) => void;
  run: <TDone>(work: () => Promise<TDone>, job: Job, from?: GenerationWork) => Promise<TDone | null>;
  cancel: () => void;
}

const IDLE: Job = { claims: false, seals: false, kept: false, steps: new Set() };

async function settle<TDone>(work: () => Promise<TDone>): Promise<Outcome<TDone>> {
  try {
    return { ok: true, done: await work() };
  } catch (failure) {
    return { ok: false, failure };
  }
}

function resumed(from: GenerationWork | undefined, job: Job): GenerationStep | null {
  const began = (from?.marks ?? []).filter((mark) => mark.step.state === "began" && job.steps.has(mark.step.step));
  return began.at(-1)?.step ?? null;
}

export function generation(
  text: Dictionary,
  call: () => Transport,
  listen: Listen,
  acknowledges = true,
): Generation {
  const [step, setStep] = createSignal<GenerationStep | null>(null);
  const [running, setRunning] = createSignal(false);
  const [cancelling, setCancelling] = createSignal(false);
  const [ended, setEnded] = createSignal<Ending | null>(null);
  const [refused, setRefused] = createSignal<Refused | null>(null);
  const [since, setSince] = createSignal<number | null>(null);
  const spent = elapsed(since);
  let round = 0;
  let job = IDLE;
  let asked = false;

  const heard = (seen: GenerationStep) => {
    if (running() && seen.state === "began" && job.steps.has(seen.step)) setStep(seen);
  };

  onMount(() => onCleanup(listen(heard)));

  const end = (how: Ending) => {
    setEnded(how);
    setRunning(false);
    setCancelling(false);
    setStep(null);
    setSince(null);
  };

  const halt = () => {
    end("halted");
    if (asked) toast("info", text.generate.cancelled);
  };

  const noticed = (began: number) => {
    if (acknowledges && job.kept) void acknowledge(call(), began).catch(() => undefined);
  };

  const drop = () => {
    round += 1;
    halt();
  };

  const run = async <TDone>(work: () => Promise<TDone>, next: Job, from?: GenerationWork): Promise<TDone | null> => {
    round += 1;
    const mine = round;
    const began = from?.began ?? Date.now();
    job = next;
    asked = false;
    setRefused(null);
    setEnded(null);
    setStep(resumed(from, next));
    setSince(began);
    setRunning(true);
    const outcome = await settle(work);
    if (mine !== round) return null;
    if (outcome.ok) {
      end("done");
      noticed(began);
      return outcome.done;
    }
    if (coded(outcome.failure) === "generate.cancelled") {
      halt();
      return null;
    }
    end("refused");
    setRefused(refusal(outcome.failure, text));
    noticed(began);
    return null;
  };

  const missed = (mine: number, answer: CancelGenerationOut) => {
    if (answer.cancelled || mine !== round || !running()) return;
    if (!job.seals) {
      drop();
      return;
    }
    setCancelling(false);
    toast("info", text.generate.late);
  };

  const unheard = (mine: number, failure: unknown) => {
    if (mine === round) setCancelling(false);
    toast("error", explain(failure) || text.generate.failed);
  };

  const cancel = () => {
    if (!running() || cancelling()) return;
    asked = true;
    if (!job.claims) {
      drop();
      return;
    }
    setCancelling(true);
    const mine = round;
    void call()("cancel_generation", {}).then(
      (answer) => missed(mine, answer),
      (failure: unknown) => unheard(mine, failure),
    );
  };

  const said = () => (cancelling() ? text.generate.cancelling : stepped(step(), text));
  const calm = () => ended() === "done" || ended() === "halted";
  const refuse = (reason: string) => setRefused({ reason, settings: false, failed: false });

  return { step, said, spent, began: since, running, cancelling, ended, calm, refused, refuse, run, cancel };
}

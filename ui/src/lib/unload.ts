import { createSignal, onCleanup } from "solid-js";

import type { OfflineStateOut } from "../ipc";
import { explain, toast } from "./toast";
import type { Transport } from "./ipc";

const STEP = 300;

export interface Where {
  bundle: string;
  topic: string | null;
}

export interface Unloading {
  job: () => string;
  state: () => OfflineStateOut | null;
  done: () => OfflineStateOut | null;
  running: () => boolean;
  broken: () => boolean;
  start: (where: Where) => void;
  stop: () => void;
}

export function unloading(call: () => Transport, broke: string, onDone?: () => void): Unloading {
  const [job, setJob] = createSignal("");
  const [state, setState] = createSignal<OfflineStateOut | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const halt = () => {
    clearInterval(timer);
    timer = undefined;
  };
  onCleanup(halt);

  const failed = (error: unknown) => {
    halt();
    toast("error", explain(error) || broke);
  };

  const look = () => {
    void (async () => {
      try {
        const out = await call()("offline_state", { job: job() });
        setState(out);
        if (!out.finished) return;
        halt();
        onDone?.();
      } catch (error) {
        setJob("");
        failed(error);
      }
    })();
  };

  const done = () => {
    const out = state();
    return out !== null && out.finished ? out : null;
  };
  const broken = () => (done()?.failed.length ?? 0) > 0;

  const start = (where: Where) => {
    const again = broken() ? job() : null;
    void (async () => {
      try {
        const out = await call()("save_offline", {
          bundle: where.bundle,
          topic: where.topic,
          again,
        });
        setJob(out.job);
        setState(null);
        halt();
        timer = setInterval(look, STEP);
        look();
      } catch (error) {
        failed(error);
      }
    })();
  };

  const stop = () => {
    void (async () => {
      try {
        await call()("stop_offline", { job: job() });
      } catch (error) {
        failed(error);
      }
    })();
  };

  return {
    job,
    state,
    done,
    running: () => job() !== "" && done() === null,
    broken,
    start,
    stop,
  };
}

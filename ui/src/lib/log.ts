import { createSignal, onMount } from "solid-js";
import type { Accessor } from "solid-js";

import type { Dictionary } from "../i18n/ru";
import type { LlmLogOut } from "../ipc";
import type { Transport } from "./ipc";
import { reason } from "./provider";
import { toast } from "./toast";

export interface Log {
  kept: Accessor<LlmLogOut | null>;
  records: Accessor<number>;
  ask: (open: boolean, clear: boolean) => void;
}

export function log(text: Dictionary, call: () => Transport): Log {
  const [kept, setKept] = createSignal<LlmLogOut | null>(null);

  const ask = (open: boolean, clear: boolean) => {
    void (async () => {
      try {
        setKept(await call()("llm_log", { open, clear }));
      } catch (error) {
        toast("error", reason(error, text));
      }
    })();
  };

  onMount(() => ask(false, false));

  return { kept, records: () => kept()?.records ?? 0, ask };
}

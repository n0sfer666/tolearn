import { createSignal, onCleanup, onMount } from "solid-js";

import type { Dictionary } from "../../i18n/ru";
import type { Transport } from "../../lib/ipc";
import { toast } from "../../lib/toast";
import { told } from "../../lib/told";

export type Phase = "starting" | "listening" | "hearing";

export interface Hold {
  field: string;
  phase: Phase;
}

export interface Voice {
  available: () => boolean;
  held: () => Hold | null;
  toggle: (field: string, put: (heard: string) => void) => Promise<void>;
  drop: (field: string) => void;
}

const refusals = (stage: Dictionary["stage"]) =>
  new Map([
    ["speech.off", stage.speechOff],
    ["speech.no-model", stage.speechNoModel],
    ["speech.unreadable", stage.speechUnreadable],
    ["speech.unsupported", stage.speechUnsupported],
    ["speech.rejected", stage.speechRejected],
    ["speech.deaf", stage.speechDeaf],
    ["speech.silent", stage.speechSilent],
    ["speech.failed", stage.speechFailed],
  ]);

export function voice(call: () => Transport, program: () => string, text: () => Dictionary): Voice {
  const [available, setAvailable] = createSignal(false);
  const [held, setHeld] = createSignal<Hold | null>(null);
  const at = () => ({ program: program() });
  const ours = (field: string) => held()?.field === field;

  const hush = () => void call()("speech_stop", at()).catch(() => undefined);

  const failed = (field: string, failure: unknown) => {
    if (!ours(field)) return;
    setHeld(null);
    toast("error", told(failure, refusals(text().stage)) || text().stage.speechFailed);
  };

  const start = async (field: string) => {
    setHeld({ field, phase: "starting" });
    try {
      await call()("speech_start", at());
    } catch (failure) {
      failed(field, failure);
      return;
    }
    if (ours(field)) setHeld({ field, phase: "listening" });
    else hush();
  };

  const stop = async (field: string, put: (heard: string) => void) => {
    setHeld({ field, phase: "hearing" });
    try {
      const out = await call()("speech_stop", at());
      if (!ours(field)) return;
      setHeld(null);
      if (out.text.trim() === "") toast("info", text().stage.unheard);
      else put(out.text);
    } catch (failure) {
      failed(field, failure);
    }
  };

  const toggle = async (field: string, put: (heard: string) => void) => {
    const hold = held();
    if (hold === null) await start(field);
    else if (hold.field === field && hold.phase === "listening") await stop(field, put);
  };

  const drop = (field: string) => {
    if (!ours(field)) return;
    setHeld(null);
    hush();
  };

  onMount(() => {
    if (program() === "") return;
    void call()("speech_state", at()).then(
      (out) => {
        setAvailable(out.available);
        if (out.listening) hush();
      },
      () => setAvailable(false),
    );
    const leave = () => {
      const hold = held();
      if (hold !== null) drop(hold.field);
    };
    window.addEventListener("pagehide", leave);
    onCleanup(() => window.removeEventListener("pagehide", leave));
  });

  return { available, held, toggle, drop };
}

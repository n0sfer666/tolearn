import { createSignal } from "solid-js";

import type { DraftView } from "../ipc";
import type { Transport } from "./ipc";

interface Terms {
  call: () => Transport;
  watch: (job: string) => void;
  broke: (error: unknown) => void;
}

export function drafting(terms: Terms) {
  const [draft, setDraft] = createSignal<DraftView | null>(null);

  const ask = (take: boolean, drop: boolean) => {
    void (async () => {
      try {
        const out = await terms.call()("generate_draft", { take, drop });
        setDraft(out.draft);
        if (out.job !== null) terms.watch(out.job);
      } catch (error) {
        terms.broke(error);
      }
    })();
  };

  return {
    draft,
    look: () => ask(false, false),
    take: () => ask(true, false),
    forget: () => ask(false, true),
    clear: () => setDraft(null),
  };
}

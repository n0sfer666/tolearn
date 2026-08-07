import { createSignal } from "solid-js";

import type { Transport } from "./ipc";

interface Terms {
  call: () => Transport;
  locale: () => string;
  today: () => string;
  job: () => string;
  open: (href: string) => void;
  close: () => void;
  broke: (error: unknown) => void;
}

export function accepting(terms: Terms) {
  const [busy, setBusy] = createSignal(false);
  const [refused, setRefused] = createSignal<string[]>([]);

  const where = async (id: string | null) => {
    const { programs } = await terms.call()("programs", { today: terms.today() });
    const card = programs.find((program) => program.id === id);
    if (card === undefined) return `/${terms.locale()}/`;
    return `/${terms.locale()}/program/?program=${encodeURIComponent(card.path)}`;
  };

  const accept = () => {
    void (async () => {
      setBusy(true);
      setRefused([]);
      try {
        const done = await terms.call()("generate_accept", {
          job: terms.job(),
          today: terms.today(),
        });
        if (!done.ok) {
          setRefused(done.violations.map((violation) => violation.message));
          return;
        }
        const href = await where(done.id);
        terms.close();
        terms.open(href);
      } catch (error) {
        terms.broke(error);
      } finally {
        setBusy(false);
      }
    })();
  };

  return { busy, refused, accept, clear: () => setRefused([]) };
}

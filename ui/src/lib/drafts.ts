import { createSignal, onCleanup, onMount } from "solid-js";

export const PAUSE_MS = 400;

type Save = (question: string, text: string) => Promise<unknown>;

export interface Drafts {
  typed: (question: string) => string | undefined;
  type: (question: string, text: string) => void;
  leave: (question: string) => Promise<void>;
  flush: () => Promise<void>;
}

export function drafts(save: Save, lost: () => void): Drafts {
  const [texts, setTexts] = createSignal<ReadonlyMap<string, string>>(new Map());
  const waiting = new Map<string, ReturnType<typeof setTimeout>>();

  const leave = async (question: string) => {
    const timer = waiting.get(question);
    if (timer === undefined) return;
    clearTimeout(timer);
    waiting.delete(question);
    try {
      await save(question, texts().get(question) ?? "");
    } catch {
      lost();
    }
  };

  const type = (question: string, text: string) => {
    setTexts(new Map(texts()).set(question, text));
    clearTimeout(waiting.get(question));
    waiting.set(
      question,
      setTimeout(() => void leave(question), PAUSE_MS),
    );
  };

  const flush = async () => {
    await Promise.all([...waiting.keys()].map(leave));
  };

  onMount(() => {
    const hide = () => void flush();
    window.addEventListener("pagehide", hide);
    onCleanup(() => {
      window.removeEventListener("pagehide", hide);
      hide();
    });
  });

  return { typed: (question) => texts().get(question), type, leave, flush };
}

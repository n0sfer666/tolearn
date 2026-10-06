import type { CommandName, Commands, GenerationStateOut, GenerationStep } from "../ipc";
import { explain, toast } from "./toast";

export type Transport = <Name extends CommandName>(
  name: Name,
  payload: Commands[Name]["input"],
) => Promise<Commands[Name]["output"]>;

export type Heard<T> = (handler: (payload: T) => void) => () => void;

export type Listen = Heard<GenerationStep>;

export type Watch = Heard<GenerationStateOut>;

const BRIDGE = "http://127.0.0.1:4319";

const bridged: Transport = async (name, payload) => {
  const answer = await fetch(BRIDGE, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ name, payload }),
  });
  const body = await answer.json();
  if (!answer.ok) throw body;
  return body;
};

const shell = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const quiet: Transport = async (name, payload) => {
  if (import.meta.env.DEV && !shell()) return bridged(name, payload);
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("command", { name, payload });
};

const heard = async <T>(event: string, handler: (payload: T) => void): Promise<() => void> => {
  const { listen } = await import("@tauri-apps/api/event");
  return listen<T>(event, (got) => handler(got.payload));
};

const tuned =
  <T>(event: string): Heard<T> =>
  (handler) => {
    if (!shell()) return () => undefined;
    const held = heard(event, handler).catch(() => () => undefined);
    return () => void held.then((stop) => stop());
  };

export const steps: Listen = tuned("generation-step");

export const states: Watch = tuned("generation-state");

export const transport: Transport = async (name, payload) => {
  try {
    return await quiet(name, payload);
  } catch (failure) {
    toast("error", explain(failure) || name);
    throw failure;
  }
};

export async function pickPackage(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const chosen = await open({
    multiple: false,
    filters: [{ name: "tolearn", extensions: ["tolearn"] }],
  });
  return typeof chosen === "string" ? chosen : null;
}

export async function pickFolder(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const chosen = await open({ directory: true, multiple: false });
  return typeof chosen === "string" ? chosen : null;
}

export function drops(handler: (paths: string[]) => void): void {
  void (async () => {
    const { getCurrentWebview } = await import("@tauri-apps/api/webview");
    await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") handler(event.payload.paths);
    });
  })();
}

import type { CommandName, Commands } from "../ipc";
import { explain, toast } from "./toast";

export type Transport = <Name extends CommandName>(
  name: Name,
  payload: Commands[Name]["input"],
) => Promise<Commands[Name]["output"]>;

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

export const transport: Transport = async (name, payload) => {
  try {
    return await quiet(name, payload);
  } catch (failure) {
    toast("error", explain(failure) || name);
    throw failure;
  }
};

export async function pick(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const chosen = await open({ directory: true, multiple: false });
  return typeof chosen === "string" ? chosen : null;
}

export async function pickArchive(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const chosen = await open({
    multiple: false,
    filters: [{ name: "archive", extensions: ["zip", "gz", "tgz"] }],
  });
  return typeof chosen === "string" ? chosen : null;
}

export async function pickFile(name: string): Promise<string | null> {
  const { save } = await import("@tauri-apps/plugin-dialog");
  const chosen = await save({
    defaultPath: name,
    filters: [{ name: "markdown", extensions: ["md"] }],
  });
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

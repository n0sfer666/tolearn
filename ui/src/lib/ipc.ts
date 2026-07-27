import type { CommandName, Commands } from "../ipc";

export type Transport = <Name extends CommandName>(
  name: Name,
  payload: Commands[Name]["input"],
) => Promise<Commands[Name]["output"]>;

export const transport: Transport = async (name, payload) => {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("command", { name, payload });
};

export async function pick(): Promise<string | null> {
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

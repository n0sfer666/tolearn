import type { Locale } from "../i18n";
import type { SettingsView } from "../ipc";
import type { Transport } from "./ipc";
import { chosen, mirrored, remember } from "./locale";
import { remember as keepTheme } from "./theme";

async function saved(call: Transport): Promise<SettingsView | null> {
  try {
    return await call("settings", { save: null });
  } catch {
    return null;
  }
}

export async function boot(call: Transport): Promise<Locale> {
  const view = await saved(call);
  if (view !== null) keepTheme(view.theme);
  const locale = chosen(view?.locale ?? null, mirrored(), navigator.language);
  remember(locale);
  return locale;
}

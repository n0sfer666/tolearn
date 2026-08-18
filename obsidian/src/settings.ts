import { App, PluginSettingTab, Setting } from "obsidian";

import type Tolearn from "./main.ts";
import { dictionary } from "./text.ts";

export interface Kept {
  data: string;
  locale: string;
}

export function defaults(): Kept {
  return { data: "", locale: "ru" };
}

export function kept(stored: unknown): Kept {
  const base = defaults();
  if (typeof stored !== "object" || stored === null) {
    return base;
  }
  const given: Record<string, unknown> = { ...stored };
  return {
    data: typeof given["data"] === "string" ? given["data"] : base.data,
    locale: typeof given["locale"] === "string" ? given["locale"] : base.locale,
  };
}

export class Settings extends PluginSettingTab {
  constructor(
    app: App,
    private readonly plugin: Tolearn,
  ) {
    super(app, plugin);
  }

  override display(): void {
    const text = dictionary(this.plugin.kept.locale);
    this.containerEl.empty();
    new Setting(this.containerEl).setName(text.plugin.data).addText((field) =>
      field.setValue(this.plugin.kept.data).onChange((value) => {
        this.plugin.kept.data = value.trim();
        void this.plugin.save();
      }),
    );
    new Setting(this.containerEl).setName(text.plugin.locale).addText((field) =>
      field.setValue(this.plugin.kept.locale).onChange((value) => {
        this.plugin.kept.locale = value.trim();
        void this.plugin.save();
      }),
    );
  }
}

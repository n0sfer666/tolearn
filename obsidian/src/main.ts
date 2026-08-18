import { MarkdownView, Plugin } from "obsidian";

import { world } from "./host.ts";
import { Settings, defaults, kept, type Kept } from "./settings.ts";
import { describe, dictionary } from "./text.ts";
import { shown, type Shown } from "./view.ts";

export default class Tolearn extends Plugin {
  kept: Kept = defaults();
  private bar: HTMLElement | null = null;

  override async onload(): Promise<void> {
    this.kept = kept(await this.loadData());
    this.bar = this.addStatusBarItem();
    this.addSettingTab(new Settings(this.app, this));
    this.addCommand({
      id: "open-topic",
      name: this.text().plugin.open,
      callback: () => this.open(),
    });
    this.registerEvent(this.app.workspace.on("file-open", () => void this.refresh()));
    this.registerEvent(this.app.workspace.on("editor-change", () => void this.refresh()));
    await this.refresh();
  }

  async save(): Promise<void> {
    await this.saveData(this.kept);
    await this.refresh();
  }

  private async refresh(): Promise<void> {
    const bar = this.bar;
    if (bar === null) {
      return;
    }
    const found = await this.current();
    bar.setText(found === null ? "" : describe(found, this.text()));
  }

  private async open(): Promise<void> {
    const found = await this.current();
    if (found !== null && found.kind === "topic") {
      window.open(found.link);
    }
  }

  private async current(): Promise<Shown | null> {
    const view = this.app.workspace.getActiveViewOfType(MarkdownView);
    const file = view?.file ?? null;
    if (file === null) {
      return null;
    }
    const note = await this.app.vault.cachedRead(file);
    return shown(note, world(this.kept.data, today()));
  }

  private text() {
    return dictionary(this.kept.locale);
  }
}

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

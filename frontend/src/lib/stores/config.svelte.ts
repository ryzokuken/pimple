import { getConfig, pickVdirRoot, setConfig, setVdirRoot } from "../ipc";
import type { AppConfig, WeekStart } from "../ipc/types";

const DEFAULT_CONFIG: AppConfig = {
  vdir_root: null,
  collection_visibility: {},
  week_start: "monday",
  window_geometry: null,
};

export class ConfigStore {
  vdirRoot = $state<string | null>(null);
  weekStart = $state<WeekStart>("monday");
  loaded = $state(false);
  pickerOpen = $state(false);
  error = $state<string | null>(null);

  /** Inflated copy of the most recent AppConfig. Mutating fields above
   *  alone is not enough — call `persist()` to flush to disk via the
   *  Tauri backend. */
  #raw: AppConfig = DEFAULT_CONFIG;

  /**
   * Load the persisted `AppConfig` from the backend. Safe to call multiple
   * times; only the first run actually fetches.
   */
  async load(): Promise<void> {
    if (this.loaded) return;
    try {
      this.#raw = await getConfig();
      this.vdirRoot = this.#raw.vdir_root;
      this.weekStart = this.#raw.week_start;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loaded = true;
    }
  }

  /**
   * First-run flow: open the native folder picker, persist the choice, and
   * apply the vdir root in the backend so the watcher starts.
   */
  async pickAndApplyVdirRoot(): Promise<void> {
    if (this.pickerOpen) return;
    this.pickerOpen = true;
    this.error = null;
    try {
      const path = await pickVdirRoot();
      if (path === null) return; // user cancelled
      await setVdirRoot(path);
      this.#raw = { ...this.#raw, vdir_root: path };
      await setConfig(this.#raw);
      this.vdirRoot = path;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.pickerOpen = false;
    }
  }

  /** Persist a week-start change. */
  async setWeekStart(value: WeekStart): Promise<void> {
    this.weekStart = value;
    this.#raw = { ...this.#raw, week_start: value };
    try {
      await setConfig(this.#raw);
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const config = new ConfigStore();

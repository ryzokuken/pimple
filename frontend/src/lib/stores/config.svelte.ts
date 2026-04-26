class ConfigStore {
  vdirRoot = $state<string | null>(null);
  weekStart = $state<"monday" | "sunday">("monday");

  /**
   * In v0.1 we read the vdir root from a CLI flag or environment variable.
   * Persistence and a UI picker land in v0.2.
   */
  async loadFromEnv(): Promise<void> {
    // The Tauri side sets this via set_vdir_root before the UI mounts;
    // here we only track it for read-only display.
  }
}

export const config = new ConfigStore();

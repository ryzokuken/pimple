import { listen } from "@tauri-apps/api/event";

export type Diagnostic = { message: string; level: string; id: number };

class DiagnosticsStore {
  list = $state<Diagnostic[]>([]);
  #counter = 0;
  #unlisten: (() => void) | null = null;

  async init(): Promise<void> {
    if (this.#unlisten) return;
    this.#unlisten = await listen<{ message: string; level: string }>(
      "diagnostic",
      (e) => {
        this.#counter += 1;
        const d = { ...e.payload, id: this.#counter };
        this.list = [...this.list, d];
        setTimeout(() => this.dismiss(d.id), 8_000);
      },
    );
  }

  destroy(): void {
    this.#unlisten?.();
    this.#unlisten = null;
  }

  dismiss(id: number): void {
    this.list = this.list.filter((d) => d.id !== id);
  }
}

export const diagnostics = new DiagnosticsStore();

import { listCollections } from "../ipc";
import type { Collection } from "../ipc/types";

class CollectionsStore {
  list = $state<Collection[]>([]);
  visible = $state<Set<string>>(new Set());
  loading = $state(false);

  async load(): Promise<void> {
    this.loading = true;
    try {
      this.list = await listCollections();
      // Default: all visible.
      this.visible = new Set(this.list.map((c) => c.id as unknown as string));
    } finally {
      this.loading = false;
    }
  }

  toggle(id: string): void {
    const next = new Set(this.visible);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    this.visible = next;
  }

  visibleIds(): string[] {
    return Array.from(this.visible);
  }
}

export const collections = new CollectionsStore();

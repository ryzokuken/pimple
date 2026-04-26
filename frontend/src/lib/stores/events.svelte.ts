import {
  eventsInRange,
  onEventsChanged,
  type EventsChangedPayload,
} from "../ipc";
import type { EventInstance } from "../ipc/types";

class EventsStore {
  instances = $state<EventInstance[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  #range: { start: string; end: string } | null = null;
  #visible: string[] = [];
  #unlisten: (() => void) | null = null;
  #refreshTimer: ReturnType<typeof setTimeout> | null = null;

  async init(): Promise<void> {
    if (this.#unlisten) return;
    this.#unlisten = await onEventsChanged((payload) =>
      this.#scheduleRefresh(payload),
    );
  }

  destroy(): void {
    this.#unlisten?.();
    this.#unlisten = null;
    if (this.#refreshTimer) clearTimeout(this.#refreshTimer);
  }

  async setView(
    start: string,
    end: string,
    visible: string[],
  ): Promise<void> {
    this.#range = { start, end };
    this.#visible = visible;
    await this.#refresh();
  }

  #scheduleRefresh(_payload: EventsChangedPayload): void {
    if (this.#refreshTimer) clearTimeout(this.#refreshTimer);
    this.#refreshTimer = setTimeout(() => {
      void this.#refresh();
    }, 100);
  }

  async #refresh(): Promise<void> {
    if (!this.#range) return;
    this.loading = true;
    this.error = null;
    try {
      this.instances = await eventsInRange(
        this.#range.start,
        this.#range.end,
        this.#visible,
      );
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }
}

export const events = new EventsStore();

import type { Page } from "@playwright/test";

/**
 * Test-time data injected into the mock Tauri IPC. Anything you don't supply
 * gets a sensible default (vdir configured, no collections, no events).
 */
export interface MockData {
  /** `null` exercises the first-run picker; anything else jumps straight
   *  to the main UI. Default: `/mock/vdir`. */
  vdirRoot?: string | null;
  /** Reply for `pick_vdir_root`. Default: `/picked/vdir`. */
  pickerResult?: string | null;
  /** Initial collections list. Default: `[]`. */
  collections?: Array<{
    id: string;
    path: string;
    display_name: string;
    color: string;
    visible: boolean;
  }>;
  /** Initial `events_in_range` result. Default: `[]`. */
  events?: Array<{
    event_uid: string;
    collection_id: string;
    summary: string;
    description: string | null;
    location: string | null;
    start: unknown;
    end: unknown;
    is_override: boolean;
    recurrence_id: unknown;
    raw_hash: string;
    is_recurring: boolean;
  }>;
}

/**
 * Inject a fake `window.__TAURI_INTERNALS__` so the frontend's `invoke()` and
 * `listen()` calls resolve to canned responses instead of crashing on the
 * missing real implementation. Recorded calls land on
 * `window.__pimpleTestCalls` for assertion.
 */
export async function setupTauriMock(page: Page, data: MockData = {}): Promise<void> {
  await page.addInitScript((d) => {
    const vdirRoot = "vdirRoot" in d ? d.vdirRoot : "/mock/vdir";
    const cfg = {
      vdir_root: vdirRoot,
      collection_visibility: {},
      week_start: "monday",
      window_geometry: null,
    };
    const collections = d.collections ?? [];
    const events = d.events ?? [];
    const calls: Array<{ cmd: string; args: unknown }> = [];

    type Handler = (args: Record<string, unknown>) => unknown;
    const handlers: Record<string, Handler> = {
      get_config: () => cfg,
      set_config: (args) => {
        Object.assign(cfg, (args["config"] as object) ?? {});
        return undefined;
      },
      list_collections: () => collections,
      events_in_range: () => events,
      pick_vdir_root: () =>
        "pickerResult" in d ? d.pickerResult : "/picked/vdir",
      set_vdir_root: () => undefined,
      create_event: () => "00000000-0000-0000-0000-000000000000",
      update_event: () => null,
      delete_event: () => undefined,
      // Tauri's @tauri-apps/api/event uses these under the hood.
      "plugin:event|listen": () => 0,
      "plugin:event|unlisten": () => undefined,
    };

    (window as unknown as { __pimpleTestCalls: typeof calls }).__pimpleTestCalls =
      calls;

    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ =
      {
        invoke: async (cmd: string, args: Record<string, unknown>) => {
          calls.push({ cmd, args });
          const h = handlers[cmd];
          if (!h) {
            throw new Error(`No mock for IPC command: ${cmd}`);
          }
          return h(args ?? {});
        },
        transformCallback: () => 0,
        unregisterCallback: () => undefined,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main", windowLabel: "main" },
        },
      };
  }, data);
}

/** Pull the recorded IPC calls out of the browser context for assertions. */
export async function recordedCalls(
  page: Page,
): Promise<Array<{ cmd: string; args: unknown }>> {
  return await page.evaluate(
    () =>
      (window as unknown as { __pimpleTestCalls: Array<{ cmd: string; args: unknown }> })
        .__pimpleTestCalls,
  );
}

import { beforeEach, describe, expect, test, vi } from "vitest";

vi.mock("../src/lib/ipc", () => ({
  getConfig: vi.fn(),
  setConfig: vi.fn(),
  pickVdirRoot: vi.fn(),
  setVdirRoot: vi.fn(),
  onEventsChanged: vi.fn(),
  listCollections: vi.fn(),
  eventsInRange: vi.fn(),
  createEvent: vi.fn(),
}));

import * as ipc from "../src/lib/ipc";
import { ConfigStore } from "../src/lib/stores/config.svelte";

const DEFAULT_CONFIG = {
  vdir_root: null as string | null,
  collection_visibility: {},
  week_start: "monday" as const,
  window_geometry: null,
};

describe("ConfigStore.load", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  test("defaults to vdirRoot=null when backend has no config", async () => {
    (ipc.getConfig as ReturnType<typeof vi.fn>).mockResolvedValue(DEFAULT_CONFIG);

    const store = new ConfigStore();
    await store.load();

    expect(store.vdirRoot).toBeNull();
    expect(store.loaded).toBe(true);
    expect(store.weekStart).toBe("monday");
  });

  test("populates vdirRoot when backend has persisted config", async () => {
    (ipc.getConfig as ReturnType<typeof vi.fn>).mockResolvedValue({
      ...DEFAULT_CONFIG,
      vdir_root: "/home/me/.calendars",
      week_start: "sunday",
    });

    const store = new ConfigStore();
    await store.load();

    expect(store.vdirRoot).toBe("/home/me/.calendars");
    expect(store.weekStart).toBe("sunday");
  });

  test("load is idempotent — only fetches once", async () => {
    (ipc.getConfig as ReturnType<typeof vi.fn>).mockResolvedValue(DEFAULT_CONFIG);

    const store = new ConfigStore();
    await store.load();
    await store.load();
    await store.load();

    expect(ipc.getConfig).toHaveBeenCalledTimes(1);
  });
});

describe("ConfigStore.pickAndApplyVdirRoot", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    (ipc.getConfig as ReturnType<typeof vi.fn>).mockResolvedValue(DEFAULT_CONFIG);
  });

  test("on user-chosen path: applies, persists, updates state", async () => {
    (ipc.pickVdirRoot as ReturnType<typeof vi.fn>).mockResolvedValue(
      "/home/me/.calendars",
    );
    (ipc.setVdirRoot as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    (ipc.setConfig as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const store = new ConfigStore();
    await store.load();
    await store.pickAndApplyVdirRoot();

    expect(ipc.pickVdirRoot).toHaveBeenCalledOnce();
    expect(ipc.setVdirRoot).toHaveBeenCalledWith("/home/me/.calendars");
    expect(ipc.setConfig).toHaveBeenCalledWith(
      expect.objectContaining({ vdir_root: "/home/me/.calendars" }),
    );
    expect(store.vdirRoot).toBe("/home/me/.calendars");
    expect(store.pickerOpen).toBe(false);
  });

  test("on user cancel: no side effects, state unchanged", async () => {
    (ipc.pickVdirRoot as ReturnType<typeof vi.fn>).mockResolvedValue(null);

    const store = new ConfigStore();
    await store.load();
    await store.pickAndApplyVdirRoot();

    expect(ipc.setVdirRoot).not.toHaveBeenCalled();
    expect(ipc.setConfig).not.toHaveBeenCalled();
    expect(store.vdirRoot).toBeNull();
  });

  test("setVdirRoot failure surfaces as error and leaves vdirRoot null", async () => {
    (ipc.pickVdirRoot as ReturnType<typeof vi.fn>).mockResolvedValue(
      "/bogus/path",
    );
    (ipc.setVdirRoot as ReturnType<typeof vi.fn>).mockRejectedValue(
      new Error("not a directory: /bogus/path"),
    );

    const store = new ConfigStore();
    await store.load();
    await store.pickAndApplyVdirRoot();

    expect(store.vdirRoot).toBeNull();
    expect(store.error).toContain("not a directory");
    expect(ipc.setConfig).not.toHaveBeenCalled();
  });

  test("concurrent calls are deduplicated via pickerOpen flag", async () => {
    let resolvePicker!: (path: string) => void;
    (ipc.pickVdirRoot as ReturnType<typeof vi.fn>).mockReturnValue(
      new Promise<string>((resolve) => {
        resolvePicker = resolve;
      }),
    );
    (ipc.setVdirRoot as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    (ipc.setConfig as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const store = new ConfigStore();
    await store.load();
    const first = store.pickAndApplyVdirRoot();
    const second = store.pickAndApplyVdirRoot();

    resolvePicker("/home/me/.calendars");
    await Promise.all([first, second]);

    expect(ipc.pickVdirRoot).toHaveBeenCalledTimes(1);
  });
});

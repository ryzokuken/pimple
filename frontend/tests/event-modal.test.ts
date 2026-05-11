import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

vi.mock("../src/lib/ipc", () => ({
  createEvent: vi.fn(),
  updateEvent: vi.fn(),
  deleteEvent: vi.fn(),
  getConfig: vi.fn(),
  setConfig: vi.fn(),
  pickVdirRoot: vi.fn(),
  setVdirRoot: vi.fn(),
  listCollections: vi.fn(),
  eventsInRange: vi.fn(),
  onEventsChanged: vi.fn(),
}));

import EventModal from "../src/lib/components/EventModal.svelte";
import * as ipc from "../src/lib/ipc";
import type { EventInstance } from "../src/lib/ipc/types";

function makeInstance(overrides: Partial<EventInstance> = {}): EventInstance {
  return {
    event_uid: "meeting-1",
    collection_id: "personal" as unknown as never,
    summary: "Daily standup",
    description: "Sync with team",
    location: "Room A",
    start: { kind: "utc", instant: "2026-04-25T15:30:00Z" },
    end: { kind: "utc", instant: "2026-04-25T16:00:00Z" },
    is_override: false,
    recurrence_id: { kind: "utc", instant: "2026-04-25T15:30:00Z" },
    raw_hash: "a".repeat(64),
    is_recurring: false,
    ...overrides,
  };
}

describe("EventModal in edit mode", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });
  afterEach(() => {
    cleanup();
  });

  test("title reads 'Edit event' when an event is passed", () => {
    const { getByRole } = render(EventModal, {
      onClose: vi.fn(),
      event: makeInstance(),
    });
    expect(getByRole("dialog").getAttribute("aria-label")).toBe("Edit event");
  });

  test("title reads 'New event' when no event is passed", () => {
    const { getByRole } = render(EventModal, { onClose: vi.fn() });
    expect(getByRole("dialog").getAttribute("aria-label")).toBe("New event");
  });

  test("Delete button is absent in create mode", () => {
    const { queryByTestId } = render(EventModal, { onClose: vi.fn() });
    expect(queryByTestId("delete-event")).toBeNull();
  });

  test("Delete button is visible in edit mode", () => {
    const { queryByTestId } = render(EventModal, {
      onClose: vi.fn(),
      event: makeInstance(),
    });
    expect(queryByTestId("delete-event")).not.toBeNull();
  });

  test("non-recurring save calls updateEvent with scope=all directly (no dialog)", async () => {
    (ipc.updateEvent as ReturnType<typeof vi.fn>).mockResolvedValue(null);
    const onClose = vi.fn();
    const { getByTestId } = render(EventModal, {
      onClose,
      event: makeInstance({ is_recurring: false }),
    });

    await fireEvent.click(getByTestId("submit-event"));
    // submit-event is type="submit" — fireEvent.click on a submit-button bubbles
    // up to the form's submit handler in jsdom/happy-dom.

    expect(ipc.updateEvent).toHaveBeenCalledOnce();
    const call = (ipc.updateEvent as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(call.scope).toBe("all");
    expect(call.uid).toBe("meeting-1");
    expect(call.expected_raw_hash).toBe("a".repeat(64));
  });

  test("non-recurring delete calls deleteEvent with scope=all directly", async () => {
    (ipc.deleteEvent as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    const onClose = vi.fn();
    const { getByTestId } = render(EventModal, {
      onClose,
      event: makeInstance({ is_recurring: false }),
    });

    await fireEvent.click(getByTestId("delete-event"));

    expect(ipc.deleteEvent).toHaveBeenCalledOnce();
    const call = (ipc.deleteEvent as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(call.scope).toBe("all");
    expect(call.occurrence).toBeNull();
  });

  test("recurring save opens the scope dialog before calling updateEvent", async () => {
    (ipc.updateEvent as ReturnType<typeof vi.fn>).mockResolvedValue(null);
    const { getByTestId, queryByTestId } = render(EventModal, {
      onClose: vi.fn(),
      event: makeInstance({ is_recurring: true }),
    });

    expect(queryByTestId("scope-this-instance")).toBeNull();
    await fireEvent.click(getByTestId("submit-event"));

    // Dialog appears; updateEvent NOT yet called.
    expect(queryByTestId("scope-this-instance")).not.toBeNull();
    expect(ipc.updateEvent).not.toHaveBeenCalled();

    // Pick a scope — updateEvent fires with that scope.
    await fireEvent.click(getByTestId("scope-this-and-future"));
    expect(ipc.updateEvent).toHaveBeenCalledOnce();
    const call = (ipc.updateEvent as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(call.scope).toBe("this_and_future");
    expect(call.occurrence).toEqual({
      kind: "utc",
      instant: "2026-04-25T15:30:00Z",
    });
  });

  test("recurring delete opens the scope dialog before calling deleteEvent", async () => {
    (ipc.deleteEvent as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    const { getByTestId, queryByTestId } = render(EventModal, {
      onClose: vi.fn(),
      event: makeInstance({ is_recurring: true }),
    });

    await fireEvent.click(getByTestId("delete-event"));
    expect(queryByTestId("scope-this-instance")).not.toBeNull();
    expect(ipc.deleteEvent).not.toHaveBeenCalled();

    await fireEvent.click(getByTestId("scope-this-instance"));
    expect(ipc.deleteEvent).toHaveBeenCalledOnce();
    const call = (ipc.deleteEvent as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(call.scope).toBe("this_instance");
  });
});

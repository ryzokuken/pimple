import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  Collection,
  CreateEventRequest,
  EventInstance,
} from "./ipc/types";

export type EventsChangedPayload =
  | { kind: "upsert"; uid: string }
  | { kind: "remove"; uid: string }
  | { kind: "full_reload" };

export async function setVdirRoot(path: string): Promise<void> {
  await invoke("set_vdir_root", { path });
}

export async function listCollections(): Promise<Collection[]> {
  return await invoke<Collection[]>("list_collections");
}

export async function eventsInRange(
  start: string,
  end: string,
  visibleCollections: string[],
): Promise<EventInstance[]> {
  return await invoke<EventInstance[]>("events_in_range", {
    start,
    end,
    visibleCollections,
  });
}

export async function createEvent(
  request: CreateEventRequest,
): Promise<string> {
  return await invoke<string>("create_event", { request });
}

export async function onEventsChanged(
  handler: (payload: EventsChangedPayload) => void,
): Promise<UnlistenFn> {
  return await listen<EventsChangedPayload>("events_changed", (e) =>
    handler(e.payload),
  );
}

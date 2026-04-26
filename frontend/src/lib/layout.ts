import type { EventInstance } from "./ipc/types";

export type LaidOutEvent = EventInstance & {
  /** Day index 0..6 within the visible week. */
  dayIndex: number;
  /** Minute-of-day for grid row start. */
  startMinute: number;
  /** Minute-of-day for grid row end. */
  endMinute: number;
  /** 0-indexed column within an overlap cluster. */
  column: number;
  /** Total columns in the cluster. */
  columns: number;
};

/**
 * Group `events` by day and assign each one a column within its overlap cluster.
 * Two events overlap when their [start, end) ranges intersect.
 */
export function layoutWeek(
  events: Array<
    EventInstance & { dayIndex: number; startMinute: number; endMinute: number }
  >,
): LaidOutEvent[] {
  const byDay: Map<number, typeof events> = new Map();
  for (const e of events) {
    const list = byDay.get(e.dayIndex) ?? [];
    list.push(e);
    byDay.set(e.dayIndex, list);
  }

  const out: LaidOutEvent[] = [];
  for (const [, list] of byDay) {
    list.sort((a, b) => a.startMinute - b.startMinute || a.endMinute - b.endMinute);
    const clusters: typeof list[] = [];
    for (const ev of list) {
      const cluster = clusters[clusters.length - 1];
      if (cluster && cluster.some((c) => c.endMinute > ev.startMinute)) {
        cluster.push(ev);
      } else {
        clusters.push([ev]);
      }
    }
    for (const cluster of clusters) {
      // Greedy column assignment.
      const columnEnds: number[] = [];
      const cols: number[] = [];
      for (const ev of cluster) {
        let placed = false;
        for (let i = 0; i < columnEnds.length; i++) {
          if (columnEnds[i]! <= ev.startMinute) {
            cols.push(i);
            columnEnds[i] = ev.endMinute;
            placed = true;
            break;
          }
        }
        if (!placed) {
          cols.push(columnEnds.length);
          columnEnds.push(ev.endMinute);
        }
      }
      const totalCols = columnEnds.length;
      for (let i = 0; i < cluster.length; i++) {
        out.push({ ...cluster[i]!, column: cols[i]!, columns: totalCols });
      }
    }
  }
  return out;
}

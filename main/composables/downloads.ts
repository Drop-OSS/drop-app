import { listen } from "@tauri-apps/api/event";
import type { DownloadableMetadata } from "~/types";

export type QueueState = {
  queue: Array<{
    meta: DownloadableMetadata;
    status: string;
    progress: number | null;
    current: number;
    max: number;
  }>;
  status: string;
};

export type StatsState = {
  speed: number; // Bytes per second
  time: number; // Seconds,
};

export const useQueueState = () =>
  useState<QueueState>("queue", () => ({ queue: [], status: "Unknown" }));

export const useStatsState = () =>
  useState<StatsState>("stats", () => ({ speed: 0, time: 0 }));

listen("update_queue", (event) => {
  const queue = useQueueState();
  queue.value = event.payload as QueueState;
});

listen("update_stats", (event) => {
  const stats = useStatsState();
  stats.value = event.payload as StatsState;
});

export const useDownloadHistory = () => useState<Array<number>>('history', () => []);

export function formatKilobytes(bytes: number): string {
  const units = ["K", "M", "G", "T", "P"];
  let value = bytes;
  let unitIndex = 0;
  const scalar = 1000;

  while (value >= scalar && unitIndex < units.length - 1) {
    value /= scalar;
    unitIndex++;
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`;
}
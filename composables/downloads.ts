import { listen } from "@tauri-apps/api/event";

export type QueueState = {
  queue: Array<{ id: string; status: string; progress: number | null }>;
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

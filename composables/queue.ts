import { listen } from "@tauri-apps/api/event";

export type QueueState = {
  queue: Array<{ id: string; status: string }>;
};

export const useQueueState = () =>
  useState<QueueState>("queue", () => ({ queue: [] }));

listen("update_queue", (event) => {
  const queue = useQueueState();
  queue.value = event.payload as QueueState;
});

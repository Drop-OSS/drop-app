import { convertFileSrc } from "@tauri-apps/api/core";

export const useObject = (id: string) => {
  return convertFileSrc(id, "object");
};

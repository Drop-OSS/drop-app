import { convertFileSrc } from "@tauri-apps/api/core";

export const useObject = async (id: string) => {
  return convertFileSrc(id, "object");
};

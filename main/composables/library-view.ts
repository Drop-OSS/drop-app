import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "~/types";

export const useLibraryView = () => {
  const libraryView = useState<"list" | "grid">("library-view", () => "list");

  // Load initial value from settings if not already loaded
  const state = libraryView.value;
  if (state === "list") {
    invoke<Settings>("fetch_settings").then((settings) => {
      if (libraryView.value === "list") {
        libraryView.value = (settings?.libraryView as "list" | "grid") || "list";
      }
    });
  }

  const toggleView = async () => {
    const newView = libraryView.value === "list" ? "grid" : "list";
    libraryView.value = newView;
    await invoke("update_settings", {
      newSettings: { libraryView: newView },
    });
  };

  return {
    libraryView,
    toggleView,
  };
};

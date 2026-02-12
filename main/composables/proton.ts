import { invoke } from "@tauri-apps/api/core";

interface ProtonPaths {
  data: Ref<{
    autodiscovered: ProtonPath[];
    custom: ProtonPath[];
    default?: string;
  }>;
  refresh: () => Promise<void>;
}

const protonPaths = useState<ProtonPaths["data"]["value"]>(
  "proton_paths",
  undefined,
);

export const useProtonPaths = async (): Promise<ProtonPaths> => {
  const refresh = async () => {
    protonPaths.value = await invoke("fetch_proton_paths");
  };
  if (protonPaths.value)
    return {
      data: protonPaths,
      refresh,
    };

  await refresh();
  return {
    data: protonPaths,
    refresh,
  };
};

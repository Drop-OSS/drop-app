import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type Game, type GameStatus as DownloadStatus, type GameStatusEnum as DownloadStatusEnum, type GameVersion, type DownloadableMetadata, DownloadableType } from "~/types";

const gameRegistry: { [key: string]: { game: Game; version?: GameVersion } } =
  {};

const downloadStatusRegistry: Map<DownloadableMetadata, Ref<DownloadStatus>> = new Map();

type OptionDownloadStatus = { [key in DownloadStatusEnum]: { version_name?: string } };
export type SerializedDownloadStatus = [
  { type: DownloadStatusEnum },
  OptionDownloadStatus | null
];

export const parseStatus = (status: SerializedDownloadStatus): DownloadStatus => {
  console.log(status);
  if (status[0]) {
    return {
      type: status[0].type,
    };
  } else if (status[1]) {
    const [[gameStatus, options]] = Object.entries(status[1]);
    return {
      type: gameStatus as DownloadStatusEnum,
      ...options,
    };
  } else {
    throw new Error("No game status");
  }
};

export const useStatus = (meta: DownloadableMetadata) => {
  return downloadStatusRegistry.get(meta)
}

export const useGame = async (gameId: string) => {
  const data: {
    game: Game;
    status: SerializedDownloadStatus;
    version?: GameVersion;
  } = await invoke("fetch_game", {
    gameId,
  });
  const meta = {
    id: gameId,
    version: data.version?.versionName,
    downloadType: DownloadableType.Game
  } satisfies DownloadableMetadata;
  if (!gameRegistry[gameId]) {

    gameRegistry[gameId] = { game: data.game, version: data.version };
  }
  if (!downloadStatusRegistry.has(meta)) {
    downloadStatusRegistry.set(meta, ref(parseStatus(data.status)));

    listen(`update_game/${gameId}`, (event) => {
      const payload: {
        status: SerializedDownloadStatus;
        version?: GameVersion;
      } = event.payload as any;

      downloadStatusRegistry.get(meta)!.value = parseStatus(payload.status);

      /**
       * I am not super happy about this.
       * 
       * This will mean that we will still have a version assigned if we have a game installed then uninstall it.
       * It is necessary because a flag to check if we should overwrite seems excessive, and this function gets called
       * on transient state updates. 
       */
      if (payload.version) {
        gameRegistry[gameId].version = payload.version;
      }
    });
  }


  const game = gameRegistry[gameId];
  const status = downloadStatusRegistry.get(meta)!;
  return { ...game, status };
};

export type FrontendGameConfiguration = {
  launchString: string;
};

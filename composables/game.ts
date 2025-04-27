import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Game, GameStatus, GameStatusEnum, GameVersion } from "~/types";

const gameRegistry: { [key: string]: { game: Game; version?: GameVersion } } =
  {};

const gameStatusRegistry: { [key: string]: Ref<GameStatus> } = {};

type OptionGameStatus = { [key in GameStatusEnum]: { version_name?: string } };
export type SerializedGameStatus = [
  { type: GameStatusEnum },
  OptionGameStatus | null
];

export const parseStatus = (status: SerializedGameStatus): GameStatus => {
  console.log(status);
  if (status[0]) {
    return {
      type: status[0].type,
    };
  } else if (status[1]) {
    const [[gameStatus, options]] = Object.entries(status[1]);
    return {
      type: gameStatus as GameStatusEnum,
      ...options,
    };
  } else {
    throw new Error("No game status");
  }
};

export const useGame = async (gameId: string) => {
  if (!gameRegistry[gameId]) {
    const data: {
      game: Game;
      status: SerializedGameStatus;
      version?: GameVersion;
    } = await invoke("fetch_game", {
      gameId,
    });
    gameRegistry[gameId] = { game: data.game, version: data.version };
    if (!gameStatusRegistry[gameId]) {
      gameStatusRegistry[gameId] = ref(parseStatus(data.status));

      listen(`update_game/${gameId}`, (event) => {
        const payload: {
          status: SerializedGameStatus;
        } = event.payload as any;
        console.log(payload.status);
        gameStatusRegistry[gameId].value = parseStatus(payload.status);
      });
    }
  }

  const game = gameRegistry[gameId];
  const status = gameStatusRegistry[gameId];
  return { ...game, status };
};

export type FrontendGameConfiguration = {
  launchString: string;
};

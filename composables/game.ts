import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Game, GameStatus, GameStatusEnum } from "~/types";

const gameRegistry: { [key: string]: Game } = {};

const gameStatusRegistry: { [key: string]: Ref<GameStatus> } = {};

type OptionGameStatus = { [key in GameStatusEnum]: { version_name?: string } };
export type SerializedGameStatus = [
  { type: GameStatusEnum },
  OptionGameStatus | null
];

const parseStatus = (status: SerializedGameStatus): GameStatus => {
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

export const useGame = async (id: string) => {
  if (!gameRegistry[id]) {
    const data: { game: Game; status: SerializedGameStatus } = await invoke(
      "fetch_game",
      {
        id,
      }
    );
    gameRegistry[id] = data.game;
    if (!gameStatusRegistry[id]) {
      gameStatusRegistry[id] = ref(parseStatus(data.status));

      listen(`update_game/${id}`, (event) => {
        const payload: {
          status: SerializedGameStatus;
        } = event.payload as any;
        gameStatusRegistry[id].value = parseStatus(payload.status);
      });
    }
  }

  const game = gameRegistry[id];
  const status = gameStatusRegistry[id];
  return { game, status };
};

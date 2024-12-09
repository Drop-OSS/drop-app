import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Game, GameStatus } from "~/types";

const gameRegistry: { [key: string]: Game } = {};

const gameStatusRegistry: { [key: string]: Ref<GameStatus> } = {};

export const useGame = async (id: string) => {
  if (!gameRegistry[id]) {
    const data: { game: Game; status: GameStatus } = await invoke(
      "fetch_game",
      {
        id,
      }
    );
    gameRegistry[id] = data.game;
    if (!gameStatusRegistry[id]) {
      gameStatusRegistry[id] = ref(data.status);

      listen(`update_game/${id}`, (event) => {
        const payload: { status: GameStatus } = event.payload as any;
        gameStatusRegistry[id].value = payload.status;
      });
    }
  }

  const game = gameRegistry[id];
  const status = gameStatusRegistry[id];
  return { game, status };
};

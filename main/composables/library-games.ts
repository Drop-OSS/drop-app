import { invoke } from "@tauri-apps/api/core";
import {
  GameStatusEnum,
  type Collection,
  type Game,
  type GameStatus,
} from "~/types";

export const useLibraryGames = () => {
  const loading = ref(false);
  const games: {
    [key: string]: { game: Game; status: Ref<GameStatus, GameStatus> };
  } = {};
  const icons: { [key: string]: string } = {};
  const covers: { [key: string]: string } = {};
  const collections: Ref<Collection[]> = ref([]);

  async function calculateGames(clearAll = false, forceRefresh = false) {
    if (clearAll) {
      collections.value = [];
      loading.value = true;
    }
    // If we update immediately, the navigation gets re-rendered before we
    // add all the necessary state, and it freaks tf out
    const newGames = await invoke<Game[]>("fetch_library", {
      hardRefresh: forceRefresh,
    });
    const otherCollections = await invoke<Collection[]>("fetch_collections", {
      hardRefresh: forceRefresh,
    });
    const allGames = [
      ...newGames,
      ...otherCollections
        .map((e) => e.entries)
        .flat()
        .map((e) => e.game),
    ].filter((v, i, a) => a.indexOf(v) === i);

    for (const game of allGames) {
      if (games[game.id]) continue;
      games[game.id] = await useGame(game.id);
    }
    for (const game of allGames) {
      if (icons[game.id]) continue;
      icons[game.id] = await useObject(game.mIconObjectId);
    }
    for (const game of allGames) {
      if (covers[game.id]) continue;
      covers[game.id] = await useObject(game.mCoverObjectId);
    }

    const libraryCollection = {
      id: "library",
      name: "Library",
      isDefault: true,
      entries: newGames.map((e) => ({ gameId: e.id, game: e })),
    } satisfies Collection;

    loading.value = false;
    collections.value = [libraryCollection, ...otherCollections];
  }

  return {
    loading: readonly(loading),
    games,
    icons,
    covers,
    collections: readonly(collections),
    calculateGames,
  };
};

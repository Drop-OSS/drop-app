import { invoke } from "@tauri-apps/api/core";
import type { Game } from "~/types";

export interface Collection {
  id: string;
  name: string;
  isDefault: boolean;
  userId: string;
  entries: CollectionEntry[];
}

export interface CollectionEntry {
  collectionId: string;
  gameId: string;
  game: Game;
}

export type Collections = Collection[];

// Registry to cache collections
const collectionsRegistry = new Map<string, Collection>();

/**
 * Fetch all collections for the current user
 */
export const useCollections = async () => {
  const collections: Collections = await invoke("fetch_collections");
  
  // Update registry
  collections.forEach(collection => {
    collectionsRegistry.set(collection.id, collection);
  });

  return collections;
};

/**
 * Fetch a single collection by ID
 */
export const useCollection = async (collectionId: string) => {
  // Check registry first
  if (collectionsRegistry.has(collectionId)) {
    return collectionsRegistry.get(collectionId)!;
  }

  const collection: Collection = await invoke("fetch_collection", {
    collectionId,
  });

  collectionsRegistry.set(collection.id, collection);
  return collection;
};

/**
 * Create a new collection
 */
export const createCollection = async (name: string) => {
  const collection: Collection = await invoke("create_collection", {
    name,
  });
  
  collectionsRegistry.set(collection.id, collection);
  return collection;
};

/**
 * Add a game to a collection
 */
export const addGameToCollection = async (collectionId: string, gameId: string) => {
  await invoke("add_game_to_collection", {
    collectionId,
    gameId,
  });

  // Refresh the collection in the registry
  const collection = await useCollection(collectionId);
  collectionsRegistry.set(collectionId, collection);
};

/**
 * Delete a collection
 */
export const deleteCollection = async (collectionId: string) => {
  const success: boolean = await invoke("delete_collection", {
    collectionId,
  });

  if (success) {
    collectionsRegistry.delete(collectionId);
  }

  return success;
};

/**
 * Remove a game from a collection
 */
export const removeGameFromCollection = async (collectionId: string, gameId: string) => {
  await invoke("delete_game_in_collection", {
    collectionId,
    gameId,
  });

  // Refresh the collection in the registry
  const collection = await useCollection(collectionId);
  collectionsRegistry.set(collectionId, collection);
}; 

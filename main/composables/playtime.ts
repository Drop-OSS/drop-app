import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { 
  GamePlaytimeStats, 
  PlaytimeUpdateEvent, 
  PlaytimeSessionStartEvent, 
  PlaytimeSessionEndEvent 
} from "~/types";

export const usePlaytime = () => {
  const playtimeStats = useState<Record<string, GamePlaytimeStats>>('playtime-stats', () => ({}));
  const activeSessions = useState<Set<string>>('active-sessions', () => new Set());

  // Fetch playtime stats for a specific game
  const fetchGamePlaytime = async (gameId: string): Promise<GamePlaytimeStats | null> => {
    try {
      const stats = await invoke<GamePlaytimeStats | null>("fetch_game_playtime", { gameId });
      if (stats) {
        playtimeStats.value[gameId] = stats;
      }
      return stats;
    } catch (error) {
      console.error(`Failed to fetch playtime for game ${gameId}:`, error);
      return null;
    }
  };

  // Fetch all playtime stats
  const fetchAllPlaytimeStats = async (): Promise<Record<string, GamePlaytimeStats>> => {
    try {
      const stats = await invoke<Record<string, GamePlaytimeStats>>("fetch_all_playtime_stats");
      playtimeStats.value = stats;
      return stats;
    } catch (error) {
      console.error("Failed to fetch all playtime stats:", error);
      return {};
    }
  };

  // Check if a session is active
  const isSessionActive = async (gameId: string): Promise<boolean> => {
    try {
      return await invoke<boolean>("is_playtime_session_active", { gameId });
    } catch (error) {
      console.error(`Failed to check session status for game ${gameId}:`, error);
      return false;
    }
  };

  // Get all active sessions
  const getActiveSessions = async (): Promise<string[]> => {
    try {
      const sessions = await invoke<string[]>("get_active_playtime_sessions");
      activeSessions.value = new Set(sessions);
      return sessions;
    } catch (error) {
      console.error("Failed to get active sessions:", error);
      return [];
    }
  };

  // Format playtime duration
  const formatPlaytime = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds}s`;
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      return `${minutes}m`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      if (minutes === 0) {
        return `${hours}h`;
      }
      return `${hours}h ${minutes}m`;
    }
  };

  // Format detailed playtime
  const formatDetailedPlaytime = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds} seconds`;
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      if (remainingSeconds === 0) {
        return `${minutes} minutes`;
      }
      return `${minutes} minutes, ${remainingSeconds} seconds`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      if (minutes === 0) {
        return `${hours} hours`;
      }
      return `${hours} hours, ${minutes} minutes`;
    }
  };

  // Format relative time (e.g., "2 hours ago")
  const formatRelativeTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffInSeconds < 60) {
      return "Just now";
    } else if (diffInSeconds < 3600) {
      const minutes = Math.floor(diffInSeconds / 60);
      return `${minutes} minute${minutes !== 1 ? 's' : ''} ago`;
    } else if (diffInSeconds < 86400) {
      const hours = Math.floor(diffInSeconds / 3600);
      return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
    } else if (diffInSeconds < 604800) {
      const days = Math.floor(diffInSeconds / 86400);
      return `${days} day${days !== 1 ? 's' : ''} ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  // Get playtime stats for a game (from cache or fetch)
  const getGamePlaytime = async (gameId: string): Promise<GamePlaytimeStats | null> => {
    if (playtimeStats.value[gameId]) {
      return playtimeStats.value[gameId];
    }
    return await fetchGamePlaytime(gameId);
  };

  // Setup event listeners
  const setupEventListeners = () => {
    // Listen for general playtime updates
    listen<PlaytimeUpdateEvent>("playtime_update", (event) => {
      const { gameId, stats, isActive } = event.payload;
      playtimeStats.value[gameId] = stats;
      
      if (isActive) {
        activeSessions.value.add(gameId);
      } else {
        activeSessions.value.delete(gameId);
      }
    });

    // Listen for session start events
    listen<PlaytimeSessionStartEvent>("playtime_session_start", (event) => {
      const { gameId } = event.payload;
      activeSessions.value.add(gameId);
    });

    // Listen for session end events
    listen<PlaytimeSessionEndEvent>("playtime_session_end", async (event) => {
      const { gameId } = event.payload;
      activeSessions.value.delete(gameId);
      
      // Refresh the game's playtime stats after session ends
      await fetchGamePlaytime(gameId);
    });
  };

  // Setup game-specific event listeners
  const setupGameEventListeners = (gameId: string) => {
    listen<PlaytimeUpdateEvent>(`playtime_update/${gameId}`, (event) => {
      const { stats, isActive } = event.payload;
      playtimeStats.value[gameId] = stats;
      
      if (isActive) {
        activeSessions.value.add(gameId);
      } else {
        activeSessions.value.delete(gameId);
      }
    });

    listen<PlaytimeSessionStartEvent>(`playtime_session_start/${gameId}`, () => {
      activeSessions.value.add(gameId);
    });

    listen<PlaytimeSessionEndEvent>(`playtime_session_end/${gameId}`, async () => {
      activeSessions.value.delete(gameId);
      
      // Refresh the game's playtime stats after session ends
      await fetchGamePlaytime(gameId);
    });
  };

  return {
    playtimeStats: readonly(playtimeStats),
    activeSessions: readonly(activeSessions),
    fetchGamePlaytime,
    fetchAllPlaytimeStats,
    isSessionActive,
    getActiveSessions,
    formatPlaytime,
    formatDetailedPlaytime,
    formatRelativeTime,
    getGamePlaytime,
    setupEventListeners,
    setupGameEventListeners,
  };
};

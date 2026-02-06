import type { Component } from "vue";

export type NavigationItem = {
  prefix: string;
  route: string;
  label: string;
};

export type QuickActionNav = {
  icon: Component;
  notifications?: number;
  action: () => Promise<void>;
};

export type User = {
  id: string;
  username: string;
  admin: boolean;
  displayName: string;
  profilePictureObjectId: string;
};

type UmuState = "Installed" | "NotInstalled" | "NotNeeded";

export type AppState = {
  status: AppStatus;
  umuState: UmuState;
  user?: User;
};

export type Game = {
  id: string;
  type: "Game" | "Executor" | "Redist";
  mName: string;
  mShortDescription: string;
  mDescription: string;
  mIconObjectId: string;
  mBannerObjectId: string;
  mCoverObjectId: string;
  mImageLibraryObjectIds: string[];
  mImageCarouselObjectIds: string[];
};

export type Collection = {
  id: string;
  name: string;
  isDefault: boolean;
  isTools?: boolean;
  entries: Array<{ gameId: string; game: Game }>;
};

export type GameVersion = {
  userConfiguration: {
    launchTemplate: string;
    overrideProtonPath: string;
  };
  setups: Array<{ platform: string }>;
  launches: Array<{ platform: string }>;
};

export enum AppStatus {
  NotConfigured = "NotConfigured",
  Offline = "Offline",
  SignedOut = "SignedOut",
  SignedIn = "SignedIn",
  SignedInNeedsReauth = "SignedInNeedsReauth",
  ServerUnavailable = "ServerUnavailable",
}

export enum GameStatusEnum {
  Remote = "Remote",
  Queued = "Queued",
  Downloading = "Downloading",
  Validating = "Validating",
  Installed = "Installed",
  Updating = "Updating",
  Uninstalling = "Uninstalling",
  SetupRequired = "SetupRequired",
  Running = "Running",
  PartiallyInstalled = "PartiallyInstalled",
}

export type GameStatus = {
  type: GameStatusEnum;
  version_name?: string;
  install_dir?: string;
};

export enum DownloadableType {
  Game = "Game",
  Tool = "Tool",
  DLC = "DLC",
  Mod = "Mod",
}

export type DownloadableMetadata = {
  id: string;
  version: string;
  downloadType: DownloadableType;
};

export type Settings = {
  autostart: boolean;
  maxDownloadThreads: number;
  forceOffline: boolean;
};

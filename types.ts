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
  profilePicture: string;
};

export type AppState = {
  status: AppStatus;
  user?: User;
};

export type Game = {
  id: string;
  mName: string;
  mShortDescription: string;
  mDescription: string;
  mIconId: string;
  mBannerId: string;
  mCoverId: string;
  mImageLibrary: string[];
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
  Installed = "Installed",
  Updating = "Updating",
  Uninstalling = "Uninstalling",
  SetupRequired = "SetupRequired",
  Running = "Running"
}

export type GameStatus = {
  type: GameStatusEnum;
  version_name?: string;
};

export enum DownloadableType {
  Game = "Game",
  Tool = "Tool",
  DLC = "DLC",
  Mod = "Mod"
}

export type DownloadableMetadata = {
  id: string,
  version: string,
  downloadType: DownloadableType
}

export type Settings = {
  autostart: boolean,
  maxDownloadThreads: number,
  forceOffline: boolean
}
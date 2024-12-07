import type { User } from "@prisma/client";
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
export type AppState = {
  status: AppStatus;
  user?: User;
};

export enum AppStatus {
  NotConfigured = "NotConfigured",
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
}

export type GameStatus = {
  type: GameStatusEnum;
  version_name?: string;
};

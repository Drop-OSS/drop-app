export type AppState = {
  status: AppStatus;
  user?: User;
};

export enum AppStatus {
  NotConfigured = "NotConfigured",
  SignedOut = "SignedOut",
  SignedIn = "SignedIn",
  SignedInNeedsReauth = "SignedInNeedsReauth",
}

export type User = {
  id: string;
  username: string;
  admin: boolean;
  email: string;
  displayName: string;
  profilePicture: string;
};

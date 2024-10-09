import type { AppState } from "~/types";

export const useAppState = () => useState<AppState>("state");
import { listen } from "@tauri-apps/api/event";
import { AppStatus, type AppState } from "~/types";

export function setupHooks() {
  const router = useRouter();

  listen("auth/processing", (event) => {
    router.push("/auth/processing");
  });

  listen("auth/failed", (event) => {
    router.push(
      `/auth/failed?error=${encodeURIComponent(event.payload as string)}`
    );
  });

  listen("auth/finished", (event) => {
    router.push("/store");
  });
}

export function initialNavigation(state: Ref<AppState>) {
  const router = useRouter();

  switch (state.value.status) {
    case AppStatus.NotConfigured:
      router.push({ path: "/setup" }).then(() => {
        console.log("Pushed Setup");
      });
      break;
    case AppStatus.SignedOut:
      router.push("/auth");
      break;
    case AppStatus.SignedInNeedsReauth:
      router.push("/auth/signedout");
      break;
    case AppStatus.ServerUnavailable:
      router.push("/error/serverunavailable");
      break;
    default:
      router.push("/store");
  }
}

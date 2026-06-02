fn main() {
    tauri_build::build();

    // Set cfg(dev) for debug builds so the code can distinguish
    // dev mode (pnpm tauri dev) from release builds (Flatpak).
    // In release builds, tauri-plugin-localhost serves frontend
    // assets via http://localhost because WebviewUrl::App does
    // not work in WebKit2GTK inside the Flatpak sandbox.
    #[cfg(debug_assertions)]
    println!("cargo:rustc-cfg=dev");
}

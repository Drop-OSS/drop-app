fn main() {
    println!("cargo::rustc-link-lib=appindicator3");
    tauri_build::build();
}

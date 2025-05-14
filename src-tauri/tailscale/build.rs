extern crate bindgen;

use abs_file_macro::abs_file;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let build_folder = PathBuf::from(abs_file!());
    let build_folder = build_folder.parent().unwrap();

    let in_path = build_folder.join("libtailscale");
    let out_path = build_folder.join("src/");

    let mut make_cmd = Command::new("make");
    make_cmd.arg("c-archive");
    make_cmd.current_dir(in_path.clone());

    make_cmd.status().expect("Make build failed");

    let bindings = bindgen::Builder::default()
        .header(in_path.join("libtailscale.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=libtailscale/tailscale.go");
    println!(
        "cargo:rustc-link-search=native={}",
        in_path.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static={}", "tailscale");
}

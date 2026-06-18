fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rustc-env=TARGET_TRIPLE={}", target);
    tauri_build::build()
}

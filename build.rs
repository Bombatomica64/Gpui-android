fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Android's packed relocations (API 23+) shrink .rela.dyn by ~1.3 MB.
    // Release only: dev builds keep plain relocations for the hot-patch tools.
    let android = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android");
    if android && std::env::var("PROFILE").as_deref() == Ok("release") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,--pack-dyn-relocs=android");
    }
}

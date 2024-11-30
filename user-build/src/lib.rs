use core::include_str;
pub fn setup_linker() {
    let linker = format!(
        r#"/**************************************************************************
* This is a linker script is generated automatically by the build script. *
* DO NOT MODIFY IT MANUALLY.                                              *
***************************************************************************/
{}"#,
        include_str!("linker.ld")
    );
    // copy the linker script to the src directory
    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_path = std::path::Path::new(&out_dir).join("linker.ld");
    std::fs::write(&linker_path, linker).unwrap();
}

pub fn setup() {
    setup_linker();
    // target
    println!("cargo:rerun-if-changed=user/linker.ld");
    println!("cargo:rustc-link-arg=-Tuser/linker.ld");
    println!("cargo:rustc-force-frame-pointers=yes");
}

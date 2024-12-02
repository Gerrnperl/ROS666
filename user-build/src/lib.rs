use core::include_str;
pub fn setup_linker(profile: &str) {
    let linker = format!(
        r#"/**************************************************************************
* This is a linker script is generated automatically by the build script. *
* DO NOT MODIFY IT MANUALLY.                                              *
***************************************************************************/
{}"#,
        if profile == "release" {
            include_str!("linker.ld")
        } else {
            include_str!("linker.debug.ld")
        }
    );
    // copy the linker script to the src directory
    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_path = std::path::Path::new(&out_dir).join(if profile == "release" {
        "linker.ld"
    } else {
        "linker.debug.ld"
    });
    std::fs::write(&linker_path, linker).unwrap();
}

pub fn setup() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    setup_linker(&profile);
    if profile == "release" {
        println!("cargo:rergun-if-changed=user/linker.ld");
        println!("cargo:rustc-link-arg=-Tuser/linker.ld");
    } else {
        println!("cargo:rerun-if-changed=user/linker.debug.ld");
        println!("cargo:rustc-link-arg=-Tuser/linker.debug.ld");
    }
    println!("cargo:rustc-force-frame-pointers=yes");
}

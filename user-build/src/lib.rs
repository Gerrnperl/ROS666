use core::include_str;

pub fn setup() {
    let linker_dir = get_linker_dir();
    clean_linkers(&linker_dir);

    let current_profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let apps = get_apps_in_order();
    for profile in &["debug", "release"] {
        for (app_id, app_name) in apps.iter().enumerate() {
            setup_linker(profile, app_name, app_id, &linker_dir);
        }
    }

    for (app_id, app_name) in apps.iter().enumerate() {
        println!(
            "cargo:rustc-link-arg-bin={app_name}=-T{linker_dir}/{linker_name}",
            app_name = app_name,
            linker_dir = linker_dir,
            linker_name = get_linker_name(&app_name, app_id, &current_profile),
        );
    }

    println!("cargo:rergun-if-changed=user-build");
    println!("cargo:rergun-if-changed=user/Cargo.toml");
    println!("cargo:rustc-force-frame-pointers=yes");
}

/// 为一个二进制目标文件生成链接脚本，
/// 生成的链接脚本将会被写入到 `linkers` 目录下
///
/// ## 参数
/// - `profile`：编译配置，`debug` 或 `release`
/// - `app_name`：二进制目标的名称
/// - `app_id`：二进制目标的 ID, 由 manifest 中的 `package.metadata.applications.order` 定义
fn setup_linker(profile: &str, app_name: &str, app_id: usize, linker_dir: &str) {
    let mut linker = format!(
        r#"/**************************************************************************
* This is a linker script is generated automatically by the build script. *
* DO NOT MODIFY IT MANUALLY.                                              *
***************************************************************************/

/* Linker script for {app_name} ({app_id}) in {profile} mode. */

{}"#,
        include_str!("linker.template.ld")
    );

    /*TEMPLATE {DEBUG-DISCARD}*/
    if profile == "release" {
        linker = replace_template(linker, "DEBUG-DISCARD", "*(.debug*)");
    } else {
        linker = replace_template(linker, "DEBUG-DISCARD", "");
    }

    let linker_path =
        std::path::Path::new(linker_dir).join(get_linker_name(app_name, app_id, profile));

    std::fs::write(&linker_path, linker).unwrap();
}

fn get_linker_dir() -> String {
    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_dir_path = std::path::Path::new(&out_dir).join(".linkers");
    linker_dir_path.to_str().unwrap().to_string()
}

fn get_linker_name(app_name: &str, app_id: usize, profile: &str) -> String {
    format!("linker-{app_id}-{app_name}-{profile}.ld")
}

fn replace_template(template: String, key: &str, value: &str) -> String {
    template.replace(&format!("/*TEMPLATE {{{}}}*/", key), value)
}

fn clean_linkers(linker_dir: &str) {
    let linker_dir = std::path::Path::new(linker_dir);
    assert!(linker_dir.ends_with(".linkers"));
    assert!(linker_dir.starts_with(std::env::var("CARGO_MANIFEST_DIR").unwrap()));
    std::fs::create_dir_all(linker_dir).unwrap();
    for entry in std::fs::read_dir(linker_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(path).unwrap();
        }
    }
}

pub fn get_apps_in_order() -> Vec<String> {
    // 应用程序顺序已在 user/Cargo.toml package.metadata.applications.order 中定义
    // Make 会获取该值
    let user_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let user_root = user_root.split("/").collect::<Vec<_>>();
    // remove the last part of the path
    let workspace_root = user_root[..user_root.len() - 1].join("/");
    let make_args = std::process::Command::new("make")
        .current_dir(workspace_root)
        .arg("echo-make-args")
        .arg("GET_MAKE_ARG=USER_BINARY_NAMES")
        .arg("--no-print-directory")
        .output()
        .unwrap()
        .stdout;
    let make_args = std::str::from_utf8(&make_args).unwrap();
    let make_args = make_args.trim().split(' ').map(|s| s.to_string());
    make_args.collect::<Vec<_>>()
}

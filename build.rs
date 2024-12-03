fn main() {
    gen_app_loader();
}

fn gen_app_loader() {
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let apps = get_apps();
    let target = std::env::var("TARGET").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let target_output_dir = std::path::Path::new(&root_dir)
        .join("target")
        .join(&target)
        .join(&profile);
    let target_output_dir = target_output_dir.to_str().unwrap();

    let count = apps.len();

    let app_table = apps
        .iter()
        .enumerate()
        .map(|(id, _)| format!("    .quad __app_{id}_start", id = id))
        .collect::<Vec<_>>()
        .join("\n");

    let app_table_end = format!("    .quad __app_{last}_end", last = count - 1);

    let app_sections = apps
        .iter()
        .enumerate()
        .map(|(id, name)| {
            gen_app_section(
                name.to_string(),
                format!("{target_output_dir}/{name}.bin"),
                id,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let app_loader = format!(
        r#".balign 4096
.section .data
.global __app_count
__app_count:
    .quad {count}
.global __app_table
__app_table:
{app_table}
{app_table_end}

{app_sections}
    "#
    );

    let app_loader_path = std::path::Path::new(&root_dir)
        .join("src")
        .join("app_loader.asm");
    std::fs::write(&app_loader_path, app_loader).unwrap();
}

fn gen_app_section(name: String, path: String, id: usize) -> String {
    format!(
        r#".section .data
.global __app_{id}_start
__app_{id}_start:
    .incbin "{path}"
.global __app_{id}_end
__app_{id}_end:
    .global __app_{id}_size
    .set __app_{id}_size, __app_{id}_end - __app_{id}_start
    .global __app_{id}_name
__app_{id}_name:
    .asciz "{name}"
    "#
    )
}

fn get_apps() -> Vec<String> {
    let make_args = std::process::Command::new("make")
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
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

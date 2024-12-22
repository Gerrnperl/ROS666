fn main() {
    println!("cargo:rergun-if-changed=src/linker.ld");
    println!("cargo:rustc-link-arg=-Tsrc/linker.ld");
}

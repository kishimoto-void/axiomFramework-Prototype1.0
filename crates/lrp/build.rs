fn main() {
    let a = include_str!("src/part_a.inc.rs");
    let b = include_str!("src/part_b.inc.rs");
    let out = std::env::var("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out).join("lrp_joined.rs");
    std::fs::write(&path, format!("{}{}", a, b)).expect("write joined");
    println!("cargo:rerun-if-changed=src/part_a.inc.rs");
    println!("cargo:rerun-if-changed=src/part_b.inc.rs");
}

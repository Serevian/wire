use std::process::Command;

fn main() {
    let shader = "shaders/shader.slang";
    let out_dir = std::env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed={shader}");

    let out = format!("{out_dir}/slang.spv");
    let status = Command::new("slangc")
        .args([
            shader,
            "-target",
            "spirv",
            "-profile",
            "spirv_1_6",
            "-emit-spirv-directly",
            "-fvk-use-entrypoint-name",
            "-entry",
            "vertMain",
            "-entry",
            "fragMain",
            "-o",
            &out,
        ])
        .status()
        .expect("Failed to run slangc");
    assert!(status.success(), "Failed to compile shaders!!!");
}

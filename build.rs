use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection_descriptor = PathBuf::from(env::var("OUT_DIR").unwrap()).join("solver_v1.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&reflection_descriptor)
        .compile(
            &[
                "proto/regnivon/v1/solver.proto",
                "proto/regnivon/v1/common.proto",
            ],
            &["googleapis", "proto"],
        )?;

    Ok(())
}

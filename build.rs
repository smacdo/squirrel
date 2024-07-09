use std::env;

use anyhow::*;
use fs_extra::{copy_items, dir::CopyOptions};

fn main() -> Result<()> {
    // Tell Cargo to re-run `build.rs` if anything in `content/` changes.
    println!("cargo:rerun-if-changed=content/*");

    // Diagnostics
    //println!("cargo:warning=CWD is {:?}", env::current_dir()?);
    //println!("cargo:warning=OUT_DIR is {:?}", env::var("OUT_DIR")?);

    // Copy the content directory to the build output directory.
    let out_dir = env::var("OUT_DIR")?;

    let copy_options = CopyOptions::new().overwrite(true);
    copy_items(&["content/"], out_dir, &copy_options)?;

    println!("files copied");

    Ok(())
}

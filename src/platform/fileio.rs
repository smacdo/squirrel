use std::path::Path;

use cfg_if::cfg_if;
use tracing::info;

/// Converts a load file path to a URL to the program's HTTP server will
/// recogonize.
#[cfg(target_arch = "wasm32")]
fn format_url<P>(file_name: P) -> anyhow::Result<reqwest::Url>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let origin = web_sys::window().unwrap().location().origin().unwrap();
    let base_url = reqwest::Url::parse(&format!("{}/", origin)).unwrap();
    let final_url = base_url.join(file_name.as_ref().to_str().unwrap())?;

    info!("url for load file request: {final_url:?}");
    Ok(final_url)
}

/// Loads a file relative to the current directory, and returns it as a string.
/// `file_path` should be relative to the content\ directory.
pub async fn load_as_string<P>(file_path: P) -> anyhow::Result<String>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    info!("load file as string: {file_path:?}",);

    cfg_if! {
      if #[cfg(target_arch = "wasm32")] {
        Ok(reqwest::get(format_url(file_path)?).await?.text().await?)
      } else {
        // TODO: This is going to break horribly when redistributing the game!
        let full_path = Path::new(env!("OUT_DIR")).join("content").join(file_path);
        Ok(std::fs::read_to_string(full_path)?)
      }
    }
}

/// Loads a file relative to the current directory, and returns it as a vector
/// of bytes. `file_path` should be relative to the content\ directory.
pub async fn load_as_binary<P>(file_path: P) -> anyhow::Result<Vec<u8>>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    info!("load file as binary: {file_path:?}");

    cfg_if! {
      if #[cfg(target_arch = "wasm32")] {
        Ok(reqwest::get(format_url(file_path)?).await?.bytes().await?.to_vec())
      } else {
        // TODO: This is going to break horribly when redistributing the game!
        let full_path = Path::new(env!("OUT_DIR")).join("content").join(file_path);
        Ok(std::fs::read(full_path)?)
      }
    }
}

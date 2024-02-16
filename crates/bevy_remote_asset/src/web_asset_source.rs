use bevy::{asset::io::PathStream, utils::BoxedFuture};
use std::path::{Path, PathBuf};

use bevy::asset::io::{AssetReader, AssetReaderError, Reader};

/// Treats paths as urls to load assets from.
pub enum WebAssetReader {
    /// Use TLS for setting up connections.
    Https,
}

impl WebAssetReader {
    fn make_uri(&self, path: &Path) -> PathBuf {
        PathBuf::from(match self {
            Self::Https => "https://",
        })
        .join(path)
    }

    /// See [bevy::asset::io::get_meta_path]
    fn make_meta_uri(&self, path: &Path) -> PathBuf {
        let mut uri = self.make_uri(path);
        let mut extension =
            path.extension().expect("asset paths must have extensions").to_os_string();
        extension.push(".meta");
        uri.set_extension(extension);
        uri
    }
}

async fn get<'a>(path: PathBuf) -> Result<Box<Reader<'a>>, AssetReaderError> {
    use std::{
        future::Future,
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    use bevy::asset::io::VecReader;
    use surf::StatusCode;

    #[pin_project::pin_project]
    struct ContinuousPoll<T>(#[pin] T);

    impl<T: Future> Future for ContinuousPoll<T> {
        type Output = T::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            // Always wake - blocks on single threaded executor.
            cx.waker().wake_by_ref();

            self.project().0.poll(cx)
        }
    }

    let str_path = path.to_str().ok_or_else(|| {
        AssetReaderError::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("non-utf8 path: {}", path.display()),
        ))
    })?;
    let mut response = ContinuousPoll(surf::get(str_path)).await.map_err(|err| {
        AssetReaderError::Io(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "unexpected status code {} while loading {}: {}",
                err.status(),
                path.display(),
                err.into_inner(),
            ),
        ))
    })?;

    match response.status() {
        StatusCode::Ok => Ok(Box::new(VecReader::new(
            ContinuousPoll(response.body_bytes())
                .await
                .map_err(|_| AssetReaderError::NotFound(path.to_path_buf()))?,
        )) as _),
        StatusCode::NotFound => Err(AssetReaderError::NotFound(path)),
        code => Err(AssetReaderError::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("unexpected status code {} while loading {}", code, path.display()),
        ))),
    }
}

impl AssetReader for WebAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(get(self.make_uri(path)))
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(get(self.make_meta_uri(path)))
    }

    fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async { Ok(false) })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(async { Err(AssetReaderError::NotFound(self.make_uri(path))) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_https_uri() {
        assert_eq!(
            WebAssetReader::Https
                .make_uri(Path::new("s3.johanhelsing.studio/dump/favicon.png"))
                .to_str()
                .unwrap(),
            "https://s3.johanhelsing.studio/dump/favicon.png"
        );
    }

    #[test]
    fn make_https_meta_uri() {
        assert_eq!(
            WebAssetReader::Https
                .make_meta_uri(Path::new("s3.johanhelsing.studio/dump/favicon.png"))
                .to_str()
                .unwrap(),
            "https://s3.johanhelsing.studio/dump/favicon.png.meta"
        );
    }
}

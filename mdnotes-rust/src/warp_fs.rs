use std::cmp;
use std::fs::Metadata;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::Poll;

use bytes::{Bytes, BytesMut};
use futures::future::Either;
use futures::{future, ready, stream, FutureExt, Stream, StreamExt};
use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt, LastModified};
use mime_guess;
use tokio::fs::File as TkFile;
use tokio::io::AsyncRead;
use urlencoding::decode;
use warp::hyper::Body;
use warp::path;
use warp::reject::{self, Rejection};
use warp::reply::Response;

pub async fn serve_file(path: &PathBuf, tail: path::Tail) -> Result<Response, Rejection> {
    let mut file_path = sanitize_path(path, tail.as_str())?;
    let is_dir = tokio::fs::metadata(file_path.clone())
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false);

    if is_dir {
        log::debug!("dir: appending index.html to directory path");
        file_path.push("index.html");
    }

    file_reply(file_path).await
}

fn sanitize_path(path: &PathBuf, tail: &str) -> Result<PathBuf, Rejection> {
    let mut buf = path.clone();
    let p = match decode(tail) {
        Ok(p) => p,
        Err(err) => {
            log::debug!("dir: failed to decode route={:?}: {:?}", tail, err);
            // FromUrlEncodingError doesn't implement StdError
            return Err(reject::not_found());
        }
    };

    for seg in p.split('/') {
        if seg.starts_with("..") {
            log::warn!("dir: rejecting segment starting with '..'");
            return Err(reject::not_found());
        } else if seg.contains('\\') {
            log::warn!("dir: rejecting segment containing with backslash (\\)");
            return Err(reject::not_found());
        } else {
            buf.push(seg);
        }
    }
    Ok(buf)
}

// // Silly wrapper since Arc<PathBuf> doesn't implement AsRef<Path> ;_;
// #[derive(Clone, Debug)]
// struct ArcPath(Arc<PathBuf>);
//
// impl AsRef<Path> for ArcPath {
//     fn as_ref(&self) -> &Path {
//         (*self.0).as_ref()
//     }
// }

async fn file_reply(path: PathBuf) -> Result<Response, Rejection> {
    let file_result = TkFile::open(path.clone()).await;

    match file_result {
        Ok(f) => file_conditional(f, path).await,
        Err(err) => {
            let rej = match err.kind() {
                io::ErrorKind::NotFound => {
                    log::debug!("file not found: {:?}", path);
                    reject::not_found()
                }
                io::ErrorKind::PermissionDenied => {
                    log::warn!("file permission denied: {:?}", path);
                    reject::not_found()
                }
                _ => {
                    log::error!("file open error (path={:?}): {} ", path, err);
                    reject::not_found()
                }
            };
            Err(rej)
        }
    }
}

async fn file_metadata(f: TkFile) -> Result<(TkFile, Metadata), Rejection> {
    match f.metadata().await {
        Ok(meta) => Ok((f, meta)),
        Err(err) => {
            log::debug!("file metadata error: {}", err);
            Err(reject::not_found())
        }
    }
}

async fn file_conditional(f: TkFile, path: PathBuf) -> Result<Response, Rejection> {
    let (file, meta) = file_metadata(f).await?;

    let len = meta.len();
    let modified = meta.modified().ok().map(LastModified::from);

    let buf_size = optimal_buf_size(&meta);
    let stream = file_stream(file, buf_size, len);
    let body = Body::wrap_stream(stream);

    let mut resp = Response::new(body);

    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    resp.headers_mut().typed_insert(ContentLength(len));
    resp.headers_mut().typed_insert(ContentType::from(mime));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());

    if let Some(last_modified) = modified {
        resp.headers_mut().typed_insert(last_modified);
    }

    Ok(resp)
}

fn file_stream(
    file: TkFile,
    buf_size: usize,
    mut len: u64,
) -> impl Stream<Item = Result<Bytes, io::Error>> + Send {
    let seek = async move { Ok(file) };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            let mut f = match result {
                Ok(f) => f,
                Err(f) => return Either::Left(stream::once(future::err(f))),
            };

            Either::Right(stream::poll_fn(move |cx| {
                reserve_at_least(&mut buf, buf_size);

                let n = match ready!(Pin::new(&mut f).poll_read_buf(cx, &mut buf)) {
                    Ok(n) => n as u64,
                    Err(err) => {
                        log::debug!("file read error: {}", err);
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if n == 0 {
                    log::debug!("file read found EOF before expected length");
                    return Poll::Ready(None);
                }

                let mut chunk = buf.split().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

fn reserve_at_least(buf: &mut BytesMut, cap: usize) {
    if buf.capacity() - buf.len() < cap {
        buf.reserve(cap);
    }
}

const DEFAULT_READ_BUF_SIZE: usize = 8_192;

fn optimal_buf_size(metadata: &Metadata) -> usize {
    let block_size = get_block_size(metadata);

    // If file length is smaller than block size, don't waste space
    // reserving a bigger-than-needed buffer.
    cmp::min(block_size as u64, metadata.len()) as usize
}

fn get_block_size(metadata: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    //TODO: blksize() returns u64, should handle bad cast...
    //(really, a block size bigger than 4gb?)

    // Use device blocksize unless it's really small.
    cmp::max(metadata.blksize() as usize, DEFAULT_READ_BUF_SIZE)
}

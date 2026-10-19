use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;

#[derive(Debug, thiserror::Error)]
#[error("http transport failed: {0}")]
pub struct HttpError(pub String);

pub type ByteStream = BoxStream<'static, Result<Vec<u8>, HttpError>>;

pub trait HttpClient: Send + Sync {
    fn post_stream(
        &self,
        url: String,
        headers: Vec<(String, String)>,
        body: serde_json::Value,
    ) -> BoxFuture<'static, Result<ByteStream, HttpError>>;
}

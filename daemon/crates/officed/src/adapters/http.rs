use futures_util::future::BoxFuture;
use futures_util::stream::StreamExt;
use inference::ports::{ByteStream, HttpClient, HttpError};

pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for ReqwestHttpClient {
    fn post_stream(
        &self,
        url: String,
        headers: Vec<(String, String)>,
        body: serde_json::Value,
    ) -> BoxFuture<'static, Result<ByteStream, HttpError>> {
        let client = self.client.clone();
        Box::pin(async move {
            let mut request = client.post(url).json(&body);
            for (name, value) in headers {
                request = request.header(name, value);
            }

            let response = request
                .send()
                .await
                .map_err(|error| HttpError(error.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let message = response.text().await.unwrap_or_default();
                return Err(HttpError(format!("{status}: {message}")));
            }

            let stream = response.bytes_stream().map(|item| {
                item.map(|bytes| bytes.to_vec())
                    .map_err(|error| HttpError(error.to_string()))
            });
            Ok(Box::pin(stream) as ByteStream)
        })
    }
}

use super::models::{ListenRequest, ListenResponse};
use super::FirestoreError;
use futures::stream::{self, Stream, StreamExt};
use reqwest_middleware::ClientWithMiddleware;
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Bytes, BytesMut};

/// A stream of `ListenResponse` messages.
pub struct ListenStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: BytesMut,
}

impl ListenStream {
    pub fn new(
        inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    ) -> Self {
        Self {
            inner,
            buffer: BytesMut::new(),
        }
    }
}

impl Stream for ListenStream {
    type Item = Result<ListenResponse, FirestoreError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 1. Try to parse a complete JSON object from the buffer.
            if let Some(len) = find_json_boundary(&self.buffer) {
                let bytes = self.buffer.split_to(len);
                let slice = &bytes[..];
                // Skip if it's just whitespace (e.g. newlines between objects)
                if slice.iter().all(|b| b.is_ascii_whitespace()) {
                    continue;
                }

                match serde_json::from_slice::<ListenResponse>(slice) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => return Poll::Ready(Some(Err(FirestoreError::SerializationError(e)))),
                }
            }

            // 2. If no complete object, poll the underlying stream for more bytes.
            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.buffer.extend_from_slice(&chunk);
                    // Loop back to try parsing again
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(FirestoreError::RequestError(e))));
                }
                Poll::Ready(None) => {
                    // End of stream.
                    if !self.buffer.is_empty() && !self.buffer.iter().all(|b| b.is_ascii_whitespace()) {
                         return Poll::Ready(Some(Err(FirestoreError::ApiError("Stream ended with incomplete JSON".into()))));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Finds the length of the first valid JSON object in the buffer.
fn find_json_boundary(buf: &[u8]) -> Option<usize> {
    if buf.is_empty() {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut started = false;
    let mut start_idx = 0;

    // Skip leading whitespace
    while start_idx < buf.len() && buf[start_idx].is_ascii_whitespace() {
        start_idx += 1;
    }

    if start_idx == buf.len() {
        return None;
    }

    for (i, &b) in buf.iter().enumerate().skip(start_idx) {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else {
            match b {
                b'{' => {
                    if !started {
                        started = true;
                    }
                    depth += 1;
                }
                b'}' => {
                    if started {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i + 1);
                        }
                    }
                }
                b'"' => {
                     if started {
                        in_string = true;
                     }
                }
                b'[' => {
                     if !started {
                        started = true;
                     }
                    depth += 1;
                }
                b']' => {
                    if started {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i + 1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    None
}


pub async fn listen_request(
    client: &ClientWithMiddleware,
    base_url: &str,
    request: &ListenRequest,
) -> Result<ListenStream, FirestoreError> {
    // The base_url passed here is usually "projects/{p}/databases/{d}".
    // The listen endpoint is at ".../documents:listen".
    let url = format!("{}/documents:listen", base_url);

    // We use a POST request with the ListenRequest in the body
    let response = client
        .post(&url)
        .json(request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(FirestoreError::ApiError(format!(
            "Listen failed {}: {}",
            status, text
        )));
    }

    // Use unfold to create a stream from response.chunk()
    let stream = stream::unfold(response, |mut resp| async move {
        match resp.chunk().await {
            Ok(Some(bytes)) => Some((Ok(bytes), resp)),
            Ok(None) => None,
            Err(e) => Some((Err(e), resp)),
        }
    });

    Ok(ListenStream::new(Box::pin(stream)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_json_boundary() {
        // Simple object
        let buf = br#"{"a":1}"#;
        assert_eq!(find_json_boundary(buf), Some(7));

        // Whitespace
        let buf = br#"  {"a":1}  "#;
        assert_eq!(find_json_boundary(buf), Some(9));

        // Nested object
        let buf = br#"{"a":{"b":2}}"#;
        assert_eq!(find_json_boundary(buf), Some(13));

        // Incomplete
        let buf = br#"{"a":1"#;
        assert_eq!(find_json_boundary(buf), None);

        // String with braces
        let buf = br#"{"a":"}"}"#;
        assert_eq!(find_json_boundary(buf), Some(9));

        // Escaped quote
        let buf = br#"{"a":"\"}"}"#;
        assert_eq!(find_json_boundary(buf), Some(11));

        // Array
        let buf = br#"{"a":[1,2]}"#;
        assert_eq!(find_json_boundary(buf), Some(11));

        // Multiple objects (should find first)
        let buf = br#"{"a":1}{"b":2}"#;
        assert_eq!(find_json_boundary(buf), Some(7));
    }
}

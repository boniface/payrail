use http::HeaderMap;

/// Raw webhook request data required for verification.
#[derive(Debug, Clone)]
pub struct WebhookRequest<'a> {
    /// Raw request body.
    pub payload: &'a [u8],
    /// Request headers.
    pub headers: HeaderMap,
}

impl<'a> WebhookRequest<'a> {
    /// Creates a webhook request.
    #[inline]
    #[must_use]
    pub const fn new(payload: &'a [u8], headers: HeaderMap) -> Self {
        Self { payload, headers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keeps_payload_and_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("webhook-id", "evt_1".parse().expect("header should parse"));

        let request = WebhookRequest::new(b"{}", headers);

        assert_eq!(request.payload, b"{}");
        assert_eq!(
            request
                .headers
                .get("webhook-id")
                .expect("header should exist"),
            "evt_1"
        );
    }
}

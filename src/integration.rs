#[cfg(feature = "axum")]
mod axum_support {
    use crate::Template;
    use axum_core::response::{IntoResponse, Response};
    use http::{header, HeaderMap, HeaderValue};

    impl IntoResponse for Template {
        fn into_response(self) -> Response {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, self.build()).into_response()
        }
    }
}

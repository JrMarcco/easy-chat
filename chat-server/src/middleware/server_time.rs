use std::{future::Future, pin::Pin};

use axum::{body::Body, extract::Request, response::Response};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use super::{X_REQUEST_ID, X_SERVER_TIME};

#[derive(Clone)]
struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start = Instant::now();
        let fut = self.inner.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let duration = format!("{} ms", start.elapsed().as_millis());

            match duration.parse() {
                Ok(dur) => {
                    res.headers_mut().append(X_SERVER_TIME, dur);
                }
                Err(e) => {
                    warn!(
                        "failed to parse server time: {} for request: {:?}",
                        e,
                        res.headers().get(X_REQUEST_ID)
                    );
                }
            }
            Ok(res)
        })
    }
}

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::CONFIG;
use axum::http::{Request, Response, StatusCode};
use futures_util::ready;
use pin_project::pin_project;
use serde::Deserialize;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TokenLayer;

impl<S> Layer<S> for TokenLayer {
    type Service = TokenMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokenMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct TokenMiddleware<S> {
    inner: S,
}

#[derive(Debug, Deserialize)]
struct Query {
    token: String,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TokenMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TokenFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let mut is_ok = false;
        if let Some(query) = request.uri().query() {
            if let Ok(query) = serde_urlencoded::from_str::<Query>(query) {
                if Some(query.token) == CONFIG.token {
                    is_ok = true;
                }
            }
        }

        let fut = self.inner.call(request);
        TokenFuture { fut, is_ok }
    }
}

#[pin_project]
pub struct TokenFuture<F> {
    #[pin]
    fut: F,
    is_ok: bool,
}

impl<F, B, E> Future for TokenFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if !this.is_ok.to_owned() {
            let mut res = Response::default();
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Poll::Ready(Ok(res));
        }
        let res = ready!(this.fut.poll(cx)?);

        Poll::Ready(Ok(res))
    }
}

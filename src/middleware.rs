use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::CONFIG;
use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
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

impl<S, ReqBody> Service<Request<ReqBody>> for TokenMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response>,
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
        let response_future = self.inner.call(request);
        TokenFuture {
            response_future,
            is_ok,
        }
    }
}

#[pin_project]
pub struct TokenFuture<F> {
    #[pin]
    response_future: F,
    is_ok: bool,
}

impl<F, E> Future for TokenFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if !this.is_ok.to_owned() {
            return Poll::Ready(Ok(StatusCode::NOT_FOUND.into_response()));
        }
        let res = ready!(this.response_future.poll(cx)?);
        Poll::Ready(Ok(res))
    }
}

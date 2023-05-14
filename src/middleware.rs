use std::marker::PhantomData;

use axum::http::{Request, Response, StatusCode};
use serde::Deserialize;
use tower_http::validate_request::ValidateRequest;

pub struct Token<ResBody> {
    token: String,
    _resbody: PhantomData<ResBody>,
}

#[derive(Debug, Deserialize)]
struct Query {
    token: String,
}

impl<ResBody> Token<ResBody> {
    pub fn new(token: String) -> Self {
        Self {
            token,
            _resbody: PhantomData,
        }
    }
}

impl<ResBody> Clone for Token<ResBody> {
    fn clone(&self) -> Self {
        Self {
            token: self.token.clone(),
            _resbody: PhantomData,
        }
    }
}

impl<B, ResBody> ValidateRequest<B> for Token<ResBody>
where
    ResBody: Default,
{
    type ResponseBody = ResBody;
    fn validate(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        if let Some(query) = request.uri().query() {
            if let Ok(query) = serde_urlencoded::from_str::<Query>(query) {
                if query.token == self.token {
                    return Ok(());
                }
            }
        }
        let mut res = Response::default();
        *res.status_mut() = StatusCode::NOT_FOUND;
        Err(res)
    }
}

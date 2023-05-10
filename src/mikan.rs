use axum::{
    body::Bytes,
    extract::{Path, RawQuery},
    http::{HeaderMap, Method},
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header::CONTENT_TYPE, StatusCode};

use crate::{util::get_header, CLIENT, CONFIG};

static MIKAN_URL: &'static str = "https://mikanani.me/";
static MIKAN_RSS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?:\/\/mikanani\.me").unwrap());

pub async fn mikan_proxy(
    path: Option<Path<String>>,
    RawQuery(query): RawQuery,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, String)> {
    let mut mikan = MIKAN_URL.to_string();
    if let Some(Path(path)) = path {
        mikan += &path;
    }
    if let Some(query) = query {
        mikan += "?";
        mikan += &query;
    }
    let res = CLIENT
        .request(method, mikan)
        .headers(headers)
        .body(body)
        .send()
        .await;
    match res {
        Ok(res) => {
            let status = res.status();
            let headers = res.headers().clone();
            let ct = get_header(&headers, CONTENT_TYPE).unwrap_or_default();
            if ct.contains("application/xml") {
                match res.text().await {
                    Ok(body) => {
                        let body = MIKAN_RSS_REGEX.replace_all(&body, &CONFIG.url).to_string();
                        let mut res = body.into_response();
                        *res.status_mut() = status;
                        *res.headers_mut() = headers;
                        Ok(res)
                    }
                    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                }
            } else {
                match res.bytes().await {
                    Ok(body) => {
                        let mut res = body.into_response();
                        *res.status_mut() = status;
                        *res.headers_mut() = headers;
                        Ok(res)
                    }
                    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                }
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

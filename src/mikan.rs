use axum::{
    body::Bytes,
    extract::{Host, Path, RawQuery},
    http::{HeaderMap, Method},
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{
    header::{ACCEPT_ENCODING, CONTENT_TYPE},
    StatusCode,
};

use crate::{util::get_header, CLIENT, CONFIG};

static MIKAN_URL: &'static str = "https://mikanani.me/";
static MIKAN_RSS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?:\/\/mikanani\.me").unwrap());

pub struct ReqwestError(reqwest::Error);

impl From<reqwest::Error> for ReqwestError {
    fn from(value: reqwest::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for ReqwestError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

pub async fn mikan_proxy(
    path: Option<Path<String>>,
    RawQuery(query): RawQuery,
    method: Method,
    Host(host): Host,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ReqwestError> {
    let mut mikan = MIKAN_URL.to_string();
    if let Some(Path(path)) = path {
        mikan += &path;
    }
    if let Some(query) = query {
        mikan += "?";
        mikan += &query;
    }
    let mut headers = headers;
    let rm_headers = ["x-forwarded-host", "cf-connecting-ip", "cf-ipcountry"];
    rm_headers.into_iter().for_each(|key| {
        headers.remove(key);
    });
    headers.insert(ACCEPT_ENCODING, "br".parse().unwrap());
    let res = CLIENT
        .request(method, mikan)
        .headers(headers)
        .body(body)
        .send()
        .await?;
    let status = res.status();
    let headers = res.headers().clone();
    let ct = get_header(&headers, CONTENT_TYPE).unwrap_or_default();
    let mut res = if ct.contains("application/xml") {
        let mut url = String::new();
        match &CONFIG.url {
            Some(s) => url += s,
            None => {
                match CONFIG.https {
                    true => url += "https",
                    false => url += "http",
                };
                url += "://";
                url += &host;
            }
        };
        let body = res.text().await?;
        let body = MIKAN_RSS_REGEX.replace_all(&body, url).to_string();
        body.into_response()
    } else {
        let body = res.bytes().await?;
        body.into_response()
    };
    *res.status_mut() = status;
    *res.headers_mut() = headers;
    Ok(res)
}

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use figment::{providers::Env, Figment};
use is_terminal::IsTerminal;
use once_cell::sync::Lazy;
use reqwest::Url;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

const PREFIX: &'static str = "MIKAN_";

pub static CONFIG: Lazy<Config> = Lazy::new(|| init_config());

#[derive(Debug)]
pub enum LogStyle {
    Auto,
    Always,
    Never,
}

impl Default for LogStyle {
    fn default() -> Self {
        Self::Auto
    }
}

impl LogStyle {
    pub fn is_color(&self) -> bool {
        match self {
            LogStyle::Auto => std::io::stdout().is_terminal(),
            LogStyle::Always => true,
            LogStyle::Never => false,
        }
    }
}

impl<'de> Deserialize<'de> for LogStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        match s.as_str() {
            "auto" => Ok(LogStyle::Auto),
            "always" => Ok(LogStyle::Always),
            "never" => Ok(LogStyle::Never),
            _ => Err(serde::de::Error::unknown_field(
                &s,
                &["auto", "always", "never"],
            )),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Log {
    pub level: String,
    pub style: LogStyle,
}

impl Default for Log {
    fn default() -> Self {
        Log {
            level: Self::level(),
            style: LogStyle::default(),
        }
    }
}

impl Log {
    fn level() -> String {
        String::from("mikan=info")
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(deserialize_with = "token_deserialize")]
    pub token: Option<String>,
    pub log: Log,
    pub addr: SocketAddr,
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: None,
            log: Log::default(),
            addr: Self::addr(),
            url: Self::url(),
        }
    }
}

impl Config {
    fn addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030)
    }
    fn url() -> String {
        "".to_string()
    }
}

fn token_deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TokenVisitor;

    impl<'de> Visitor<'de> for TokenVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("token must be a string.")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(std::str::from_utf8(v).unwrap().to_owned()))
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_owned()))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Some(s))
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }
    deserializer.deserialize_any(TokenVisitor)
}

pub fn init_config() -> Config {
    let config = Figment::from(Env::prefixed(PREFIX))
        .merge(Env::prefixed(PREFIX).split("_"))
        .extract::<Config>();
    match config {
        Ok(mut config) => {
            if config.url.is_empty() {
                config.url = format!("http://{}", config.addr);
            }
            if let Err(e) = Url::parse(&config.url) {
                panic!("Failed to parse an absolute URL, Reason: {}", e.to_string());
            };
            println!("{:#?}", config);
            config
        }
        Err(err) => {
            panic!("{:?}", err);
        }
    }
}

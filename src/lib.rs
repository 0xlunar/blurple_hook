use chrono::prelude::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::usize;
use anyhow::format_err;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    #[serde(skip)]
    webhook_url: String,
    content: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    embeds: Vec<Embed>,
    components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Component {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    title: Option<String>,
    #[serde(rename = "type")]
    _type: String,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<String>,
    color: Option<usize>,
    footer: Option<Footer>,
    image: Option<Image>,
    thumbnail: Option<Thumbnail>,
    video: Option<Video>,
    provider: Option<Provider>,
    author: Option<Author>,
    fields: Vec<Field>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Footer {
    text: String,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Image {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Thumbnail {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Video {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Provider {
    name: Option<String>,
    url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Author {
    name: String,
    url: Option<String>,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    name: String,
    value: String,
    inline: bool,
}
pub enum ColourType<S: AsRef<str>> {
    Hex(S),
    Integer(usize),
}

impl Webhook {
    pub fn new<S: AsRef<str>>(webhook_url: S) -> Webhook {
        Webhook {
            webhook_url: webhook_url.as_ref().to_string(),
            content: None,
            username: None,
            avatar_url: None,
            embeds: Vec::new(),
            components: Vec::new(),
        }
    }
    pub fn set_content<S: AsRef<str>>(mut self, content: S) -> Self {
        self.content = Some(content.as_ref().to_string());
        self
    }
    pub fn set_username<S: AsRef<str>>(mut self, username: S) -> Self {
        self.username = Some(username.as_ref().to_string());
        self
    }
    pub fn set_avatar_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.avatar_url = Some(url.as_ref().to_string());
        self
    }
    pub fn add_embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }
    pub async fn send(&self) -> anyhow::Result<()> {
        let client = reqwest::Client::new();

        let body = serde_json::to_string(self).unwrap();

        let resp = client
            .post(format!("{}?wait=true", &self.webhook_url))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        match resp.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => {
                Ok(())
            },
            _ => {
                let body = resp.text().await.unwrap_or(String::from(""));
                Err(format_err!("Failed to send request, {}", body))
            }
        }
    }
}

impl Embed {
    pub fn new() -> Embed {
        Embed {
            title: None,
            _type: "rich".to_string(),
            description: None,
            url: None,
            timestamp: None,
            color: None,
            footer: None,
            image: None,
            thumbnail: None,
            video: None,
            provider: None,
            author: None,
            fields: Vec::new(),
        }
    }
    pub fn set_title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.title = Some(title.as_ref().to_string());
        self
    }
    pub fn set_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());
        self
    }
    pub fn set_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = Some(url.as_ref().to_string());
        self
    }
    pub fn set_timestamp(mut self, timestamp: Option<&std::time::SystemTime>) -> Self {
        let timestamp: DateTime<Utc> = match timestamp {
            Some(ts) => (*ts).into(),
            None => Utc::now(),
        };
        self.timestamp = Some(timestamp.format("%+").to_string());
        self
    }
    pub fn set_colour<S: AsRef<str>>(mut self, colour: ColourType<S>) -> Self {
        let colour: usize = match colour {
            ColourType::Hex(hex) => usize::from_str_radix(
                hex.as_ref()
                    .trim_start_matches('#')
                    .trim_start_matches("0x"),
                16,
            )
                .unwrap_or(10066329),
            ColourType::Integer(int) => int,
        };

        self.color = Some(colour);
        self
    }
    pub fn set_color<S: AsRef<str>>(self, color: ColourType<S>) -> Self {
        self.set_colour(color)
    }
    pub fn set_footer<S: AsRef<str>>(
        mut self,
        text: S,
        icon_url: Option<S>,
        proxy_icon_url: Option<S>,
    ) -> Self {
        self.footer = Some(Footer {
            text: text.as_ref().to_string(),
            icon_url: icon_url.map(|url| url.as_ref().to_owned()),
            proxy_icon_url: proxy_icon_url.map(|url| url.as_ref().to_owned()),
        });

        self
    }
    pub fn set_image<S: AsRef<str>>(
        mut self,
        url: S,
        proxy_url: Option<S>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.image = Some(Image {
            url: url.as_ref().to_string(),
            proxy_url: proxy_url.map(|url| url.as_ref().to_owned()),
            height,
            width,
        });

        self
    }
    pub fn set_thumbnail<S: AsRef<str>>(
        mut self,
        url: S,
        proxy_url: Option<S>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.thumbnail = Some(Thumbnail {
            url: url.as_ref().to_string(),
            proxy_url: proxy_url.map(|url| url.as_ref().to_owned()),
            height,
            width,
        });

        self
    }
    pub fn set_video<S: AsRef<str>>(
        mut self,
        url: S,
        proxy_url: Option<S>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.video = Some(Video {
            url: url.as_ref().to_string(),
            proxy_url: proxy_url.map(|url| url.as_ref().to_owned()),
            height,
            width,
        });

        self
    }
    pub fn set_provider(mut self, name: Option<String>, url: Option<String>) -> Self {
        self.provider = Some(Provider { name, url });
        self
    }
    pub fn set_author<S: AsRef<str>>(
        mut self,
        name: S,
        url: Option<S>,
        icon_url: Option<S>,
        proxy_icon_url: Option<S>,
    ) -> Self {
        self.author = Some(Author {
            name: name.as_ref().to_string(),
            url: url.map(|url| url.as_ref().to_owned()),
            icon_url: icon_url.map(|url| url.as_ref().to_owned()),
            proxy_icon_url: proxy_icon_url.map(|url| url.as_ref().to_owned()),
        });

        self
    }
    pub fn add_field<S: AsRef<str>>(mut self, name: S, value: S, inline: bool) -> Self {
        let field = Field {
            name: name.as_ref().to_string(),
            value: value.as_ref().to_string(),
            inline,
        };

        self.fields.push(field);

        self
    }

    pub fn add_fields<A: AsRef<str>, B: AsRef<str>>(mut self, fields: &mut Vec<Field>) -> Self {
        self.fields.append(fields);
        self
    }
}
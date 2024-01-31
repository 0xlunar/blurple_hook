use chrono::prelude::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::usize;
use anyhow::format_err;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Webhook<'a> {
    #[serde(skip)]
    webhook_url: &'a str,
    content: Option<&'a str>,
    username: Option<&'a str>,
    avatar_url: Option<&'a str>,
    embeds: Vec<Embed<'a>>,
    components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Component {}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Embed<'a> {
    title: Option<&'a str>,
    #[serde(rename = "type")]
    _type: &'a str,
    description: Option<&'a str>,
    url: Option<&'a str>,
    timestamp: Option<String>,
    color: Option<usize>,
    footer: Option<Footer<'a>>,
    image: Option<Image<'a>>,
    thumbnail: Option<Thumbnail<'a>>,
    video: Option<Video<'a>>,
    provider: Option<Provider<'a>>,
    author: Option<Author<'a>>,
    fields: Vec<Field<'a>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Footer<'a> {
    text: &'a str,
    icon_url: Option<&'a str>,
    proxy_icon_url: Option<&'a str>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Image<'a> {
    url: &'a str,
    proxy_url: Option<&'a str>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Thumbnail<'a> {
    url: &'a str,
    proxy_url: Option<&'a str>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Video<'a> {
    url: &'a str,
    proxy_url: Option<&'a str>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Provider<'a> {
    name: Option<&'a str>,
    url: Option<&'a str>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Author<'a> {
    name: &'a str,
    url: Option<&'a str>,
    icon_url: Option<&'a str>,
    proxy_icon_url: Option<&'a str>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Field<'a> {
    name: &'a str,
    value: &'a str,
    inline: bool,
}
pub enum ColourType<'a> {
    Hex(&'a str),
    Integer(usize),
}

impl<'a> Webhook<'a> {
    pub fn new(webhook_url: &str) -> Webhook {
        Webhook {
            webhook_url,
            content: None,
            username: None,
            avatar_url: None,
            embeds: Vec::new(),
            components: Vec::new(),
        }
    }
    pub fn set_content(mut self, content: &'a str) -> Self {
        self.content = Some(content);
        self
    }
    pub fn set_username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    }
    pub fn set_avatar_url(mut self, url: &'a str) -> Self {
        self.avatar_url = Some(url);
        self
    }
    pub fn add_embed(mut self, embed: Embed<'a>) -> Self {
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

impl<'a> Embed<'a> {
    pub fn new() -> Embed<'a> {
        Embed {
            title: None,
            _type: "rich",
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
    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn set_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn set_url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }
    pub fn set_timestamp(mut self, timestamp: Option<&std::time::SystemTime>) -> Self {
        let timestamp: DateTime<Utc> = match timestamp {
            Some(ts) => (*ts).into(),
            None => Utc::now(),
        };
        let timestamp = timestamp.format("%+").to_string();
        self.timestamp = Some(timestamp);
        self
    }
    pub fn set_colour(mut self, colour: ColourType<'a>) -> Self {
        let colour: usize = match colour {
            ColourType::Hex(hex) => usize::from_str_radix(
                hex
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
    pub fn set_color(self, color: ColourType<'a>) -> Self {
        self.set_colour(color)
    }
    pub fn set_footer(
        mut self,
        text: &'a str,
        icon_url: Option<&'a str>,
        proxy_icon_url: Option<&'a str>,
    ) -> Self {
        self.footer = Some(Footer {
            text,
            icon_url,
            proxy_icon_url,
        });

        self
    }
    pub fn set_image(
        mut self,
        url: &'a str,
        proxy_url: Option<&'a str>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.image = Some(Image {
            url,
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_thumbnail(
        mut self,
        url: &'a str,
        proxy_url: Option<&'a str>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.thumbnail = Some(Thumbnail {
            url,
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_video(
        mut self,
        url: &'a str,
        proxy_url: Option<&'a str>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        self.video = Some(Video {
            url,
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_provider(mut self, name: Option<&'a str>, url: Option<&'a str>) -> Self {
        self.provider = Some(Provider { name, url });
        self
    }
    pub fn set_author(
        mut self,
        name: &'a str,
        url: Option<&'a str>,
        icon_url: Option<&'a str>,
        proxy_icon_url: Option<&'a str>,
    ) -> Self {
        self.author = Some(Author {
            name,
            url,
            icon_url,
            proxy_icon_url,
        });

        self
    }
    pub fn add_field(mut self, name: &'a str, value: &'a str, inline: bool) -> Self {
        let field = Field {
            name,
            value,
            inline,
        };

        self.fields.push(field);

        self
    }

    pub fn add_fields(mut self, fields: &mut Vec<Field<'a>>) -> Self {
        self.fields.append(fields);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::{Author, ColourType, Embed, Field, Footer, Thumbnail, Webhook};

    #[test]
    fn create_embed() {
        let embed = Embed::new()
            .set_colour(ColourType::Hex("#FFFFFF"))
            .set_author("Author Name", Some("https://example.com/"), None, None)
            .set_thumbnail("https://example.com/", None, None, None)
            .set_title("Example")
            .set_url("https://example.com/")
            .set_footer("Footer Text", None, None)
            .set_description("Description Text")
            .add_field("Example 1", "Value 1", true)
            .add_fields(&mut vec![
                Field {
                    name: "Example 2",
                    value: "Value 2",
                    inline: false,
                },
                Field {
                    name: "Example 3",
                    value: "Value 3",
                    inline: false,
                }
            ]);

        let expected = Embed {
            title: Some("Example"),
            _type: "rich",
            description: Some("Description Text"),
            url: Some("https://example.com/"),
            timestamp: None,
            color: Some(16777215),
            footer: Some(Footer {
                text: "Footer Text",
                icon_url: None,
                proxy_icon_url: None,
            }),
            image: None,
            thumbnail: Some(Thumbnail {
                url: "https://example.com/",
                proxy_url: None,
                height: None,
                width: None,
            }),
            video: None,
            provider: None,
            author: Some(Author {
                name: "Author Name",
                url: Some("https://example.com/"),
                icon_url: None,
                proxy_icon_url: None,
            }),
            fields: vec![
                Field {
                    name: "Example 1",
                    value: "Value 1",
                    inline: true,
                },
                Field {
                    name: "Example 2",
                    value: "Value 2",
                    inline: false,
                },
                Field {
                    name: "Example 3",
                    value: "Value 3",
                    inline: false,
                }
            ],
        };
        assert_eq!(embed, expected);
    }

    #[test]
    fn embed_adds_to_webhook() {
        let webhook = Webhook::new("https://discord.com/webhook")
            .set_content("Content Text")
            .set_username("Test Username");

        let embed = Embed::new()
            .set_colour(ColourType::Hex("#FFFFFF"))
            .set_author("Author Name", Some("https://example.com/"), None, None)
            .set_thumbnail("https://example.com/", None, None, None)
            .set_title("Example")
            .set_url("https://example.com/")
            .set_footer("Footer Text", None, None)
            .set_description("Description Text")
            .add_field("Example 1", "Value 1", true)
            .add_fields(&mut vec![
                Field {
                    name: "Example 2",
                    value: "Value 2",
                    inline: false,
                },
                Field {
                    name: "Example 3",
                    value: "Value 3",
                    inline: false,
                }
            ]);

        let webhook = webhook.add_embed(embed);

        let expected = Webhook {
            webhook_url: "https://discord.com/webhook",
            content: Some("Content Text"),
            username: Some("Test Username"),
            avatar_url: None,
            embeds: vec![
                Embed {
                    title: Some("Example"),
                    _type: "rich",
                    description: Some("Description Text"),
                    url: Some("https://example.com/"),
                    timestamp: None,
                    color: Some(16777215),
                    footer: Some(Footer {
                        text: "Footer Text",
                        icon_url: None,
                        proxy_icon_url: None,
                    }),
                    image: None,
                    thumbnail: Some(Thumbnail {
                        url: "https://example.com/",
                        proxy_url: None,
                        height: None,
                        width: None,
                    }),
                    video: None,
                    provider: None,
                    author: Some(Author {
                        name: "Author Name",
                        url: Some("https://example.com/"),
                        icon_url: None,
                        proxy_icon_url: None,
                    }),
                    fields: vec![
                        Field {
                            name: "Example 1",
                            value: "Value 1",
                            inline: true,
                        },
                        Field {
                            name: "Example 2",
                            value: "Value 2",
                            inline: false,
                        },
                        Field {
                            name: "Example 3",
                            value: "Value 3",
                            inline: false,
                        }
                    ],
                }
            ],
            components: vec![],
        };

        assert_eq!(webhook, expected);
    }

    #[tokio::test]
    async fn submit_webhook() {

        let webhook_url = env::var("WEBHOOK").unwrap();
        let webhook = Webhook::new(&webhook_url);

        let embed = Embed::new()
            .set_title("Blurple Test")
            .set_description("Testing Blurple")
            .set_url("https://example.com/")
            .set_timestamp(None)
            .add_field("Example 1", "Value 1", true)
            .add_fields(&mut vec![
                Field {
                    name: "Example 2",
                    value: "Value 2",
                    inline: true,
                },
                Field {
                    name: "Example 3",
                    value: "Value 3",
                    inline: false,
                },
                Field {
                    name: "Example 4",
                    value: "Example 4",
                    inline: false,
                }
            ]);

        let webhook = webhook.add_embed(embed);
        let result = webhook.send().await;

        assert!(result.is_ok());
    }
}
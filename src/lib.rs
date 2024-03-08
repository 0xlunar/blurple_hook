use chrono::prelude::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::usize;
use anyhow::format_err;

#[cfg(feature = "queue")]
pub mod queue {
    use std::collections::VecDeque;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    use tokio::task::JoinHandle;
    use tokio::time::Instant;
    use crate::Webhook;

    pub struct WebhookQueue {
        pub webhooks: Arc<Mutex<VecDeque<Webhook>>>,
    }

    impl WebhookQueue {
        pub fn new() -> Self {
            Self {
                webhooks: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        pub async fn enqueue(queue: Arc<Mutex<VecDeque<Webhook>>>, webhook: Webhook) {
            let mut q = queue.lock().await;
            q.push_front(webhook);
        }

        pub async fn enqueue_multi(queue: Arc<Mutex<VecDeque<Webhook>>>, webhooks: Vec<Webhook>) {
            let mut q = queue.lock().await;
            for webhook in webhooks {
                q.push_front(webhook);
            }
        }

        pub fn start(self) -> JoinHandle<Self> {
            tokio::task::spawn(async move {
                loop {
                    let (one, two) = {
                        let mut whs = self.webhooks.as_ref().lock().await;
                        let one = whs.pop_back();
                        let two = whs.pop_back();
                        (one, two)
                    };

                    if cfg!(test) {
                        if one.is_none() && two.is_none() {
                            return self;
                        }
                    }

                    // only 2 so sequential is fine
                    let _ = match one {
                        Some(w) => w.send().await,
                        None => Ok(()),
                    };

                    let _ = match two {
                        Some(w) => w.send().await,
                        None => Ok(()),
                    };

                    // we can send 2 webhooks every 2 seconds,
                    tokio::time::sleep_until(Instant::now() + Duration::from_millis(2000)).await;
                }
            })
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Webhook {
    #[serde(skip)]
    webhook_url: String,
    content: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    embeds: Vec<Embed>,
    components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Component {}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Footer {
    text: String,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Image {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Thumbnail {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Video {
    url: String,
    proxy_url: Option<String>,
    height: Option<usize>,
    width: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Provider {
    name: Option<String>,
    url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct Author {
    name: String,
    url: Option<String>,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: bool,
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
        let timestamp = timestamp.format("%+").to_string();
        self.timestamp = Some(timestamp);
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
    pub fn set_footer<A: AsRef<str>, B: AsRef<str>, C: AsRef<str>>(
        mut self,
        text: A,
        icon_url: Option<B>,
        proxy_icon_url: Option<C>,
    ) -> Self {
        let icon_url = icon_url.map(|n| n.as_ref().to_string());
        let proxy_icon_url = proxy_icon_url.map(|n| n.as_ref().to_string());

        self.footer = Some(Footer {
            text: text.as_ref().to_string(),
            icon_url,
            proxy_icon_url,
        });

        self
    }
    pub fn set_image<A: AsRef<str>, B: AsRef<str>>(
        mut self,
        url: A,
        proxy_url: Option<B>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let proxy_url = proxy_url.map(|p| p.as_ref().to_string());

        self.image = Some(Image {
            url: url.as_ref().to_string(),
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_thumbnail<A: AsRef<str>, B: AsRef<str>>(
        mut self,
        url: A,
        proxy_url: Option<B>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let proxy_url = proxy_url.map(|p| p.as_ref().to_string());

        self.thumbnail = Some(Thumbnail {
            url: url.as_ref().to_string(),
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_video<A: AsRef<str>, B: AsRef<str>>(
        mut self,
        url: A,
        proxy_url: Option<B>,
        height: Option<usize>,
        width: Option<usize>,
    ) -> Self {
        let proxy_url = proxy_url.map(|p| p.as_ref().to_string());

        self.video = Some(Video {
            url: url.as_ref().to_string(),
            proxy_url,
            height,
            width,
        });

        self
    }
    pub fn set_provider<A: AsRef<str>, B: AsRef<str>>(mut self, name: Option<A>, url: Option<B>) -> Self {
        let name = name.map(|n| n.as_ref().to_string());
        let url = url.map(|n| n.as_ref().to_string());
        self.provider = Some(Provider { name, url });
        self
    }
    pub fn set_author<A: AsRef<str>, B: AsRef<str>, C: AsRef<str>, D: AsRef<str>>(
        mut self,
        name: A,
        url: Option<B>,
        icon_url: Option<C>,
        proxy_icon_url: Option<D>,
    ) -> Self {
        let url = url.map(|n| n.as_ref().to_string());
        let icon_url = icon_url.map(|n| n.as_ref().to_string());
        let proxy_icon_url = proxy_icon_url.map(|n| n.as_ref().to_string());
        self.author = Some(Author {
            name: name.as_ref().to_string(),
            url,
            icon_url,
            proxy_icon_url,
        });

        self
    }
    pub fn add_field<A: AsRef<str>, B: AsRef<str>>(mut self, name: A, value: B, inline: bool) -> Self {
        let field = Field {
            name: name.as_ref().to_string(),
            value: value.as_ref().to_string(),
            inline,
        };

        self.fields.push(field);

        self
    }

    pub fn add_fields(mut self, fields: &mut Vec<Field>) -> Self {
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
            .set_author("Author Name", Some("https://example.com/"), None::<String>, None::<String>)
            .set_thumbnail("https://example.com/", None::<String>, None, None)
            .set_title("Example")
            .set_url("https://example.com/")
            .set_footer("Footer Text", None::<String>, None::<String>)
            .set_description("Description Text")
            .add_field("Example 1", "Value 1", true)
            .add_fields(&mut vec![
                Field {
                    name: "Example 2".to_string(),
                    value: "Value 2".to_string(),
                    inline: false,
                },
                Field {
                    name: "Example 3".to_string(),
                    value: "Value 3".to_string(),
                    inline: false,
                }
            ]);

        let expected = Embed {
            title: Some("Example".to_string()),
            _type: "rich".to_string(),
            description: Some("Description Text".to_string()),
            url: Some("https://example.com/".to_string()),
            timestamp: None,
            color: Some(16777215),
            footer: Some(Footer {
                text: "Footer Text".to_string(),
                icon_url: None,
                proxy_icon_url: None,
            }),
            image: None,
            thumbnail: Some(Thumbnail {
                url: "https://example.com/".to_string(),
                proxy_url: None,
                height: None,
                width: None,
            }),
            video: None,
            provider: None,
            author: Some(Author {
                name: "Author Name".to_string(),
                url: Some("https://example.com/".to_string()),
                icon_url: None,
                proxy_icon_url: None,
            }),
            fields: vec![
                Field {
                    name: "Example 1".to_string(),
                    value: "Value 1".to_string(),
                    inline: true,
                },
                Field {
                    name: "Example 2".to_string(),
                    value: "Value 2".to_string(),
                    inline: false,
                },
                Field {
                    name: "Example 3".to_string(),
                    value: "Value 3".to_string(),
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
            .set_author("Author Name", Some("https://example.com/"), None::<String>, None::<String>)
            .set_thumbnail("https://example.com/", None::<String>, None, None)
            .set_title("Example")
            .set_url("https://example.com/")
            .set_footer("Footer Text", None::<String>, None::<String>)
            .set_description("Description Text")
            .add_field("Example 1", "Value 1", true)
            .add_fields(&mut vec![
                Field {
                    name: "Example 2".to_string(),
                    value: "Value 2".to_string(),
                    inline: false,
                },
                Field {
                    name: "Example 3".to_string(),
                    value: "Value 3".to_string(),
                    inline: false,
                }
            ]);

        let webhook = webhook.add_embed(embed);

        let expected = Webhook {
            webhook_url: "https://discord.com/webhook".to_string(),
            content: Some("Content Text".to_string()),
            username: Some("Test Username".to_string()),
            avatar_url: None,
            embeds: vec![
                Embed {
                    title: Some("Example".to_string()),
                    _type: "rich".to_string(),
                    description: Some("Description Text".to_string()),
                    url: Some("https://example.com/".to_string()),
                    timestamp: None,
                    color: Some(16777215),
                    footer: Some(Footer {
                        text: "Footer Text".to_string(),
                        icon_url: None,
                        proxy_icon_url: None,
                    }),
                    image: None,
                    thumbnail: Some(Thumbnail {
                        url: "https://example.com/".to_string(),
                        proxy_url: None,
                        height: None,
                        width: None,
                    }),
                    video: None,
                    provider: None,
                    author: Some(Author {
                        name: "Author Name".to_string(),
                        url: Some("https://example.com/".to_string()),
                        icon_url: None,
                        proxy_icon_url: None,
                    }),
                    fields: vec![
                        Field {
                            name: "Example 1".to_string(),
                            value: "Value 1".to_string(),
                            inline: true,
                        },
                        Field {
                            name: "Example 2".to_string(),
                            value: "Value 2".to_string(),
                            inline: false,
                        },
                        Field {
                            name: "Example 3".to_string(),
                            value: "Value 3".to_string(),
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
                    name: "Example 2".to_string(),
                    value: "Value 2".to_string(),
                    inline: true,
                },
                Field {
                    name: "Example 3".to_string(),
                    value: "Value 3".to_string(),
                    inline: false,
                },
                Field {
                    name: "Example 4".to_string(),
                    value: "Example 4".to_string(),
                    inline: false,
                }
            ]);

        let webhook = webhook.add_embed(embed);
        let result = webhook.send().await;

        assert!(result.is_ok());
    }
    #[cfg(feature = "queue")]
    #[tokio::test]
    async fn test_queue() {
        use crate::queue::WebhookQueue;
        use std::sync::Arc;

        let queue = WebhookQueue::new();

        let webhook_url = env::var("WEBHOOK").unwrap();

        let webhooks = Arc::clone(&queue.webhooks);
        for i in 0..5 {
            let webhooks = Arc::clone(&webhooks);
            let embed = Embed::new().set_title("Example");
            let webhook = Webhook::new(&webhook_url).add_embed(embed);

            WebhookQueue::enqueue(webhooks, webhook).await;
        }

        {
            let webhooks = Arc::clone(&webhooks);
            let webhooks = webhooks.lock().await;
            assert_eq!(webhooks.len(), 5, "Len is not 5, {}", webhooks.len());
        }

        let _ = queue.start().await;

        let webhooks = Arc::clone(&webhooks);
        let webhooks = webhooks.lock().await;
        assert!(webhooks.is_empty(), "Webhooks not empty, {}", webhooks.len())
    }
}
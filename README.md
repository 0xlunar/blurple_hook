# Blurple Hook
Discord Webhook implementation built in **Rust**.

## Notice
This is a personal library used for my own projects, This is only intended to work for me, but feel free to fork and make your own versions as you please if this doesn't suit your needs.

This package will probably work for other services that implement similar systems, such as Slack, but is only intended to support Discord currently.

## Installation
`cargo add blurple_hook`

## Example

```rust
use blurple_hook::{Webhook, Embed, Field, ColourType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let webhook = Webhook::new("https://discord.com/webhook/")
            .set_username("Blurple Hook")
            .set_content("Example Content");
    
    let embed = Embed::new()
            .set_title("Example")
            .set_timestamp(None)
            .set_url("https://example.com/")
            .set_colour(ColourType::Hex("#5865F2"))
            .add_fields(vec![
                Field {
                    name: "Field Title 1",
                    value: "Field Value 1",
                    inline: true,
                },
                Field {
                    name: "Field Title 2",
                    value: "Field Value 2",
                    inline: true,
                }
            ]);
    
    let webhook = webhook.add_embed(embed);
    webhook.send().await?;
    
    Ok(())
}
```

Some methods such as set_colour have both spellings available for their method names (ie `set_colour`and `set_color`), however types are in Australian/British English spelling for now.

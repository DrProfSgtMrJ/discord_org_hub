use dioxus::prelude::*;

#[component]
pub fn DownloadButton() -> Element {
    rsx! {
        div { class: "download-container",
            h2 { class: "download-title", "🤖 Download the Discord Bot" }
            p { class: "download-description",
                "Get our Discord bot and enhance your server experience! "
                "The bot includes game management, leaderboard tracking, and more."
            }

            a {
                href: "https://discord.com/oauth2/authorize?client_id=1466997290819649619&permissions=0&integration_type=0&scope=bot",
                class: "download-button",
                target: "_blank",
                rel: "noopener noreferrer",

                div { class: "button-content",
                    span { class: "download-icon", "⬇️" }
                    span { class: "download-text", "Download Bot" }
                }
            }

            p { class: "download-note",
                "Note: Installation instructions will be provided with the download"
            }
        }
    }
}

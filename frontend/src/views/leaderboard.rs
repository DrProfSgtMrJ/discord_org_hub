use crate::components::Leaderboard;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

#[component]
pub fn LeaderboardPage() -> Element {
    let nav = use_navigator();

    rsx! {
        div { class: "leaderboard-page",
            div { class: "leaderboard-header",
                h1 { "🏆 Server Leaderboard" }
                p { "Track the top performers and compete for the top spot!" }
                button {
                    class: "back-button",
                    onclick: move |_| { nav.push("/"); },
                    "← Back to Home"
                }
            }
            Leaderboard {}
        }
    }
}

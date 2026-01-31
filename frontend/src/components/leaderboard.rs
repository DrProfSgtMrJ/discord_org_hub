use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Player {
    pub discord_name: String,
    pub win_rate: f32,
    pub currently_playing_field: String,
}

#[component]
pub fn Leaderboard() -> Element {
    let players = use_signal(|| {
        vec![
            Player {
                discord_name: "Player#1234".to_string(),
                win_rate: 75.5,
                currently_playing_field: "Field A".to_string(),
            },
            Player {
                discord_name: "Gamer#5678".to_string(),
                win_rate: 68.2,
                currently_playing_field: "Field B".to_string(),
            },
            Player {
                discord_name: "Pro#9999".to_string(),
                win_rate: 82.1,
                currently_playing_field: "Field C".to_string(),
            },
        ]
    });

    rsx! {
        div { class: "leaderboard-container",
            h2 { class: "leaderboard-title", "🏆 Leaderboard" }

            div { class: "leaderboard-table-container",
                table { class: "leaderboard-table",
                    thead {
                        tr {
                            th { "Rank" }
                            th { "Discord Name" }
                            th { "Win Rate" }
                            th { "Currently Playing Field" }
                        }
                    }
                    tbody {
                        {players().iter().enumerate().map(|(index, player)| {
                            rsx! {
                                tr { class: "leaderboard-row",
                                    td { class: "rank-cell", "{index + 1}" }
                                    td { class: "name-cell", "{player.discord_name}" }
                                    td { class: "winrate-cell", "{player.win_rate:.1}%" }
                                    td { class: "field-cell", "{player.currently_playing_field}" }
                                }
                            }
                        })}
                    }
                }
            }

            if players().is_empty() {
                div { class: "empty-leaderboard",
                    p { "No players on the leaderboard yet. Be the first to compete!" }
                }
            }
        }
    }
}

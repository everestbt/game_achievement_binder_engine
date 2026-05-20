use super::App;

use crate::{Credentials, Message};

use iced::font;
use iced::widget::{
    table, text, center_x, center_y, column, scrollable, image
};
use iced::{Center, Left, Font, Element};
use db::{
    achievement_store, 
};
use api::game_fetch;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Goal {
    // DISPLAY
    pub game_name: String,
    pub display_name: String,
    pub description: String,
    // DATA
    pub app_id: i32,
    pub achievement_name: String,
}

impl Goal {
    pub async fn list(credentials: Credentials) -> Vec<Self> {
        let game_map = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await
            .into_iter()
            .map(|g| (g.appid, g))
            .collect::<HashMap<_, _>>();

        let mut goals = achievement_store::get_achievements().expect("Failed to load achievements");
        goals.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
        goals.iter().map(|g| Goal {
                game_name: game_map.get(&g.app_id).unwrap().name.clone(),
                display_name: g.display_name.clone(),
                description: g.description.clone().unwrap_or("-".to_string()),
                app_id: g.app_id,
                achievement_name: g.achievement_name.clone(),
            })
            .collect()
    }
}

impl App {
    pub fn goal_view(&self) -> Element<'_, Message> {
        let main_view = if let Some(goals) = &self.goals {
            {
                let bold = |header| {
                    text(header).font(Font {
                        weight: font::Weight::Bold,
                        ..Font::DEFAULT
                    })
                };
                let columns = [
                    table::column(bold("Icon"), |goal: &Goal| 
                        {
                            if let Some(i) = self.goal_icons.get(&(goal.app_id, goal.achievement_name.clone())) {
                                column![image(i).width(60).height(60)]
                            }
                            else {
                                column![text("loading")]
                            }
                        }
                        )
                        .align_x(Left)
                        .align_y(Center),
                    table::column(bold("Game Name"), |goal: &Goal| text(&goal.game_name)),
                    table::column(bold("Achievement Name"), |goal: &Goal| text(&goal.display_name))
                        .align_x(Left)
                        .align_y(Center),
                    table::column(bold("Description"), |goal: &Goal| text(&goal.description))
                        .align_x(Left)
                        .align_y(Center),
                ];

                column![table(columns, goals)
                    .padding_x(10)
                    .padding_y(5)
                    .separator_x(1)
                    .separator_y(1)]
            }
        } 
        else {
            column![
                text("Loading")
            ]
        };

        column![
            center_y(scrollable(center_x(main_view)).spacing(10)).padding(10),
        ].into()
    }
}

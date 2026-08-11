use super::App;

use crate::{
    Message, 
    OWNED_GAMES
};

use iced::font;
use iced::widget::{
    table, text, center_x, center_y, column, scrollable, image
};
use iced::{Center, Left, Font, Element};
use module::{
    Module,
    GameIdentifier,
    achievements::{
        ModuleGoal,
        get_goals,
    }
};

#[derive(Debug, Clone)]
pub struct Goal {
    // DISPLAY
    pub game_name: String,
    pub display_name: String,
    pub description: String,
    // DATA
    pub id: GameIdentifier,
    pub achievement_name: String,
}

impl Goal {
    pub async fn list(modules: Vec<Module>) -> Vec<Self> {
        let mut goals = Vec::new();
        for m in modules {
            let mut module_goals = get_goals(&m).expect("Failed to load achievements");
            module_goals.sort();
            let mut mapped: Vec<Goal> = module_goals.iter().map(|g| 
                match g {
                    ModuleGoal::STEAM(a) => {
                        let id = GameIdentifier { module: m.clone(), id: a.game_id };
                        Goal {
                            game_name: OWNED_GAMES.get(&id).unwrap().name.clone(),
                            display_name: a.display_name.clone(),
                            description: a.description.clone().unwrap_or("-".to_string()),
                            id,
                            achievement_name: a.achievement_name.clone(),
                        }
                    }
                })
                .collect();
            goals.append(&mut mapped);
        }
        goals
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
                            if let Some(i) = self.goal_icons.get(&(goal.id.clone(), goal.achievement_name.clone())) {
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

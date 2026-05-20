use super::App;

use crate::View;
use crate::Message;
use crate::Credentials;

use db::excluded_achievement_store;
use iced::widget::{
    center_x, center_y, column, text, button, table, scrollable, image, image::Handle
};
use iced::{Center, Left, Element, Font, font};
use api::{
    achievement_fetch,
    achievement_fetch::GameAchievement,
    game_fetch,
    game_fetch::Game,
};
use std::collections::{HashSet, HashMap};
use db::{
    game_target_store,
    achievement_store,
};
use rayon::prelude::*;
use goals_lib::goals;
use simple_error::SimpleError;

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub app_id: i32,
    pub game_name: String,
    pub target: bool,
    pub complete: bool,
    pub goals: Vec<GameGoalDisplay>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum GoalState {
    Goal,
    Incomplete,
    Complete,
    Excluded,
}

#[derive(Debug, Clone)]
pub struct GameGoalDisplay {
    // DISPLAY
    display_name: String,
    description: String,
    // DATA
    pub goal_state: GoalState,
    pub achievement_name: String,
    pub icon: String,
    pub icon_gray: String,
}

impl App {
    pub fn game_view(&self) -> Element<'_, Message> {
        match self.view {
            View::Game(app_id) => {
                if let Some(game) = self.game_views.get(&app_id) {
                    let game_target_button = {
                        if !game.target {
                            Some(button("Target!").on_press(Message::SetAsGameTarget(app_id)))
                        }
                        else if !game.complete {
                            Some(button("Set as complete!").on_press(Message::SetGameAsComplete(app_id)))
                        }
                        else {
                            None
                        }
                    };
                    let random_achievement = button("Random achievement!").on_press(Message::GenerateRandomAchievement(app_id));

                    let controls = if let Some(target) = game_target_button {
                        column![
                            center_x(target),
                            center_x(random_achievement),
                        ]
                    }
                    else {
                        column![random_achievement]
                    };

                    let table = {
                        let bold = |header| {
                            text(header).font(Font {
                                weight: font::Weight::Bold,
                                ..Font::DEFAULT
                            })
                        };
                        let columns = [
                            table::column(bold("Icon"), |goal: &GameGoalDisplay| 
                                {
                                    if let Some(i) = self.goal_icons.get(&(app_id, goal.achievement_name.clone())) {
                                        column![image(i).width(60).height(60)]
                                    }
                                    else {
                                        column![text("loading")]
                                    }
                                }
                                )
                                .align_x(Left)
                                .align_y(Center),
                            table::column(bold("Achievement"), |goal: &GameGoalDisplay| text(&goal.display_name).style({
                                match goal.goal_state {
                                    GoalState::Complete => text::success,
                                    GoalState::Incomplete => text::default,
                                    GoalState::Goal => text::warning,
                                    GoalState::Excluded => text::danger,
                                }
                            }))
                                .align_x(Left)
                                .align_y(Center),
                            table::column(bold("Description"), |goal: &GameGoalDisplay| text(&goal.description))
                                .align_x(Left)
                                .align_y(Center),
                            table::column(bold("Exclude"), |goal: &GameGoalDisplay| button("Exclude").on_press(Message::ExcludeAchievement(app_id, goal.achievement_name.clone())))
                                .align_x(Left)
                                .align_y(Center),
                        ];

                        table(columns, &game.goals)
                            .padding_x(10)
                            .padding_y(5)
                            .separator_x(1)
                            .separator_y(1)
                    };

                    column![
                        center_x(text(game.game_name.clone())),
                        center_x(controls),
                        center_y(scrollable(center_x(table)).spacing(10)).padding(10),
                    ].into()
                }
                else {
                    column![
                        text("Loading")
                    ].into()
                }
            },
            _ => unreachable!("Only called when a game view")
        }
    }

    pub fn handle_generated_random_achievement(&mut self, game: Game, random_achievement: Option<GameAchievement>) {
        if let Some(ra) = random_achievement {
            achievement_store::save_achievement(&ra.name, &ra.display_name, &ra.description, &game.appid, &game.last_played).expect("Failed to save achievement");
            if let Some(game_view) = self.game_views.get_mut(&game.appid) {
                if let Some(achievement) = game_view.goals.iter_mut().find(|a| a.achievement_name == ra.name) {
                    achievement.goal_state = GoalState::Goal;
                }
            }
        }
    }
}

pub async fn generate_random_achievement(credentials: Credentials, app_id: i32) -> Result<(Game, Option<GameAchievement>), SimpleError> {
    if let Some(game) = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await.iter().find(|g| g.appid == app_id) {
        Ok((game.clone(), goals::get_random_achievement_for_game(&credentials.key, &credentials.steam_id, game).await))
    }
    else {
        Err(SimpleError::new("No game with that app_id"))
    }
}

pub async fn load_game_display(credentials: Credentials, app_id: i32, game_name: String) -> GameDisplay {
    let player_achievements = achievement_fetch::get_player_achievements(&credentials.key, &credentials.steam_id, &app_id).await;   
    let excluded_achievements: HashSet<String> = excluded_achievement_store::get_excluded_achievements_for_app(&app_id).expect("Failed to load excluded achievements")
        .iter()
        .map(|a| a.achievement_name.clone())
        .collect();

    let mut goals: Vec<GameGoalDisplay> = achievement_fetch::get_game_achievements(&credentials.key, &app_id).await
        .par_iter()
        .map(|a| {
            let goal_state = {
                if player_achievements.as_ref()
                    .map(|p| p.achievements.iter()
                        .find(|pa| pa.apiname == a.name)
                        .map(|pa| pa.achieved == 1))
                        .unwrap_or(None)
                        .unwrap_or(false) {
                    GoalState::Complete
                }
                else if excluded_achievements.contains(&a.name) {
                    GoalState::Excluded
                }
                else if achievement_store::get_achievements_for_app(&app_id).expect("Failed to read achievement store").iter().any(|goal| goal.achievement_name == a.name) {
                    GoalState::Goal
                }
                else {
                    GoalState::Incomplete
                }
            };

            GameGoalDisplay {
                display_name : a.display_name.clone(),
                description: a.description.clone().unwrap_or("-".to_string()),
                goal_state,
                achievement_name: a.name.clone(),
                icon: a.icon.clone(),
                icon_gray: a.icongray.clone(),
            }
        })
        .collect();
    goals.sort_by_key(|g| g.goal_state);
    let target = game_target_store::get_game_target(&app_id).expect("Failed to load target");
    GameDisplay { 
        app_id,
        game_name,
        goals,
        target: target.is_some(),
        complete: target.map(|t| t.complete).unwrap_or(false),
    }
}

pub async fn load_all_goal_icons(app_id: i32, achievements: Vec<GameGoalDisplay>) -> HashMap<(i32, String), Handle> {
    let mut map = HashMap::new();
    for a in achievements {
        if let Ok(r) = load_goal_icon(app_id, a.achievement_name, a.icon, a.icon_gray, a.goal_state).await {
            map.insert((r.0, r.1), r.2);
        }
        // This drops the error, it will reload on a fresh request
    }
    map
}

pub async fn load_goal_icon(app_id: i32, achievement_name: String, icon_url: String, icon_gray_url: String, goal_state: GoalState) -> Result<(i32, String, Handle), SimpleError> {
    let img_response = if goal_state == GoalState::Complete {
        reqwest::get(icon_url).await
    }
    else {
        reqwest::get(icon_gray_url).await
    };
    if let Ok(r) = img_response {
        if let Ok(b) = r.bytes().await {
            Ok((app_id, achievement_name, Handle::from_bytes(b)))
        }
        else {
            Err(SimpleError::new("Failed to read bytes"))
        }
    }
    else {
        Err(SimpleError::new("Failed to reach url"))
    }
    
} 
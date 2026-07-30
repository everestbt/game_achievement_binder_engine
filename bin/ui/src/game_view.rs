use super::App;

use crate::OWNED_GAMES;
use crate::View;
use crate::Message;
use crate::Credentials;

use iced::widget::{
    center_x, center_y, column, text, button, table, scrollable, image, image::Handle, row, text_input
};
use iced::{Center, Left, Element, Font, font};
use std::collections::{HashSet, HashMap};
use rayon::prelude::*;
use simple_error::{SimpleError, SimpleResult};
use module::{
    Module, 
    Game, 
    GameAchievement, 
    get_random_achievement_for_game, 
    get_game_achievements,
    game_cover::get_game_cover_url,
    game_targets::{
        get_game_target_status,
        TargetStatus,
    },
    achievements::{
        ModuleGoal,
        get_game_goals,
        get_excluded_achievements,
        save_achievement_goal,
    },
};
use steam_utils::SteamAchievement;

#[derive(Debug, Clone)]
pub struct GameDisplay {
    pub app_id: i32,
    pub game_name: String,
    pub target: bool,
    pub complete: bool,
    pub goals: Vec<GameGoalDisplay>,
    // view state
    pub game_cover_edit: bool,
    pub game_cover_url: String,
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
    pub icon: Option<String>,
    pub icon_gray: Option<String>,
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

                    let controls: Element<'_, Message> = if let Some(target) = game_target_button {
                        column![
                            center_x(target),
                            center_x(random_achievement),
                        ].into()
                    }
                    else {
                        column![random_achievement].into()
                    };

                    let table: Element<'_, Message> = {
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
                            .into()
                    };

                    let game_cover: Element<'_, Message> = if let Some(cover) = self.game_covers.get(&game.app_id) {
                        image(cover).width(600).height(1100).into()
                    }
                    else {
                        text("No cover loaded").into()
                    };

                    let game_cover_edit: Element<'_, Message> = if game.game_cover_edit {
                        text_input("Enter a url for the game cover", &game.game_cover_url)
                                .on_input(Message::GameCoverURLInput)
                                .on_submit(Message::SaveGameCover)
                                .into()
                    }
                    else {
                        button("Edit game cover").on_press(Message::EditGameCover(game.app_id)).into()
                    };
                    
                    row! [
                        column![
                            center_x(game_cover),
                            center_x(text(game.game_name.clone())),
                            center_x(controls),
                            center_x(game_cover_edit)
                        ],
                        column![
                            center_y(scrollable(center_x(table)).spacing(10)).padding(10),
                        ]
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
            let steam_achievement = ModuleGoal::STEAM(SteamAchievement { 
                achievement_name: ra.id.clone(), 
                display_name: ra.display_name.clone(), 
                description: ra.description.clone(), 
                game_id: game.id.clone(), 
                last_played: game.last_played.to_epoch_days() as i64 * 86400 
            });
            save_achievement_goal(steam_achievement).expect("Failed to save achievement");
            if let Some(game_view) = self.game_views.get_mut(&game.id) && let Some(achievement) = game_view.goals.iter_mut().find(|a| a.achievement_name == ra.id) {
                achievement.goal_state = GoalState::Goal;
            }
        }
    }
}

pub async fn generate_random_achievement(credentials: Credentials, app_id: i32) -> SimpleResult<(Game, Option<GameAchievement>)> {
    if let Some(game) = OWNED_GAMES.get(&app_id) {
        Ok((game.clone(), get_random_achievement_for_game(Module::STEAM(credentials.key, credentials.steam_id), Some(game.id)).await))
    }
    else {
        Err(SimpleError::new("No game with that app_id"))
    }
}

pub async fn load_game_display(credentials: Credentials, app_id: i32, game_name: String) -> GameDisplay {
    let steam_module = Module::STEAM(credentials.key.clone(), credentials.steam_id.clone());
    let excluded_achievements: HashSet<String> = get_excluded_achievements(&steam_module, &app_id).expect("Failed to load excluded achievements");
    let steam_module = Module::STEAM(credentials.key, credentials.steam_id);

    let mut goals: Vec<GameGoalDisplay> = get_game_achievements(steam_module.clone(), Some(app_id)).await
        .par_iter()
        .map(|a| {
            let goal_state = {
                if a.achieved {
                    GoalState::Complete
                }
                else if excluded_achievements.contains(&a.id) {
                    GoalState::Excluded
                }
                else if get_game_goals(&steam_module, &app_id).expect("Failed to read goals").iter().any(|goal| {
                    match goal {
                        ModuleGoal::STEAM(steam_achievement) => *steam_achievement.achievement_name == a.id
                    }
                }) {
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
                achievement_name: a.id.clone(),
                icon: a.achieved_icon_id.clone(),
                icon_gray: a.unachieved_icon_id.clone(),
            }
        })
        .collect();
    goals.sort_by_key(|g| g.goal_state);
    let target = get_game_target_status(&steam_module, &app_id).expect("Failed to load game target");
    let game_cover_url = get_game_cover_url(steam_module, &app_id).expect("Failed to load game cover").map_or("".to_string(), |g| g);
    GameDisplay { 
        app_id,
        game_name,
        goals,
        target: target.is_some(),
        complete: target.map(|t| match t {
                TargetStatus::Complete => true,
                _ => false,
            }).unwrap_or(false),
        game_cover_edit: false,
        game_cover_url,
    }
}

pub async fn load_all_goal_icons(app_id: i32, achievements: Vec<GameGoalDisplay>) -> HashMap<(i32, String), Handle> {
    let mut map = HashMap::new();
    for a in achievements {
        if let Some(achieved_icon) = a.icon && let Some(unachieved_icon) = a.icon_gray && let Ok(r) = load_goal_icon(app_id, a.achievement_name, achieved_icon, unachieved_icon, a.goal_state).await {
            map.insert((r.0, r.1), r.2);
        }
        // This drops the error, it will reload on a fresh request
    }
    map
}

pub async fn load_goal_icon(app_id: i32, achievement_name: String, icon_url: String, icon_gray_url: String, goal_state: GoalState) -> SimpleResult<(i32, String, Handle)> {
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
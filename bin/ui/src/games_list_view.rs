use super::App;

use crate::{Message, OWNED_GAMES};

use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button, checkbox, text_input
};
use iced::{Element, Font};
use db::{
    game_completion_cache,
    game_completion_cache::GameCompletion,
    game_target_store,
};
use api::{
    game_fetch::Game,
};
use std::collections::{HashMap, HashSet};
use std::cmp::Reverse;
use rayon::prelude::*;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum GameListFilter {
    Targets,
    #[default]
    InProgress,
    Completed,
    Perfected,
}

#[derive(Debug, Clone)]
pub struct GameListDisplay {
    //DISPLAY
    pub game_name: String,
    pub progress_display: String,
    //DATA
    pub id: i32,
}

#[derive(Debug, Clone)]
pub struct GameListResult {
    pub filter: GameListFilter,
    pub has_achievements: bool,
    pub list: Vec<GameListDisplay>,
}

impl GameListDisplay {
    pub async fn list(has_achievements: bool, filter: GameListFilter, title_search: Option<String>) -> GameListResult {
        let completed_games_cache: HashMap<i32, GameCompletion> = game_completion_cache::get_game_completion()
            .expect("Failed to load completed games")
            .iter()
            .map(|n| (n.app_id, n.clone()))
            .collect();
        let target_set: HashSet<i32> = game_target_store::get_game_targets().expect("Failed to load targets")
            .iter()
            .filter(|t| !t.complete)
            .map(|t| t.app_id)
            .collect();

        let owned_games_vec: Vec<&Game> = OWNED_GAMES.values().collect();
        let mut list: Vec<(&&Game, i8)> = owned_games_vec
            .par_iter()
            .filter(|g| {
                if let Some(search) = &title_search {
                    g.name.to_uppercase().contains(search.to_uppercase().as_str())
                }
                else {
                    true
                }
            })
            .filter(|g| {
                match filter {
                    GameListFilter::Targets => {
                        target_set.contains(&g.appid)
                    }
                    GameListFilter::InProgress => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) < 100 || target_set.contains(&g.appid)
                    },
                    GameListFilter::Completed => {
                        completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0) == 100 && !target_set.contains(&g.appid)
                    },
                    GameListFilter::Perfected => {
                        completed_games_cache.get(&g.appid).map(|c| c.perfect).unwrap_or(false) && !target_set.contains(&g.appid)
                    }
                }
            })
            .filter(|g| {
                if has_achievements {
                    completed_games_cache.get(&g.appid).map(|c| c.has_achievements).unwrap_or(false)
                }
                else {
                    true
                }
            })
            .map(|g| (g, completed_games_cache.get(&g.appid).map(|c| c.complete).unwrap_or(0))) // Game, Progress
            .collect();
        list.sort_by_key(|a| Reverse(a.1));

        GameListResult {
            filter,
            has_achievements,
            list: list
                .par_iter()
                .map(|g| {
                    GameListDisplay{
                        game_name: g.0.name.clone(),
                        progress_display: g.1.to_string(),
                        id: g.0.appid,
                    }
                })
                .collect()
        }
    }
}

impl App {
    pub fn game_list_view(&self, filter: GameListFilter) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("Targets").on_press(Message::GamesView(GameListFilter::Targets)),
                button("In progress").on_press(Message::GamesView(GameListFilter::InProgress)),
                button("Completed").on_press(Message::GamesView(GameListFilter::Completed)),
                button("Perfected").on_press(Message::GamesView(GameListFilter::Perfected)),
            ]
        };

        let random_game = button("Random Game").on_press(Message::RandomGame);

        let achievement_filter = checkbox(self.games_have_achievements_filter)
            .label("Has Achievements")
            .on_toggle(Message::AchievementCheckboxToggled);
        // Check if game list for selection ahs loaded
        let game_list = self.games.get(&(filter, self.games_have_achievements_filter));
        
        let game_count = if let Some(games) = game_list {
            text("Number of games:".to_owned() + games.len().to_string().as_str())
        }
        else {
            text("Loading...")
        };
        
        let game_search: Element<'_, Message> = text_input("Type something here...", &self.game_list_search)
                .on_input(Message::GameListSearch)
                .into();
        
        let main_view = {
            if let Some (games) = game_list {
                let bold = |header| {
                    text(header).font(Font {
                        weight: font::Weight::Bold,
                        ..Font::DEFAULT
                    })
                };
                let columns = [
                    table::column(bold("Game Name"), |game: &GameListDisplay| button(game.game_name.as_str()).on_press(Message::GameView(game.id))),
                    table::column(bold("Progress"), |game: &GameListDisplay| text(game.progress_display.as_str())),
                ];

                column![
                    table(columns, games)
                        .padding_x(10)
                        .padding_y(5)
                        .separator_x(1)
                        .separator_y(1)
                ]
            }
            else {
                column![text("Loading game list...")]
            }
        };
        column![
            center_x(filter_games).padding(5),
            center_x(achievement_filter).padding(5),
            center_x(random_game).padding(5),
            center_x(game_count).padding(5),
            center_x(game_search).padding(5),
            center_y(scrollable(center_x(main_view)).spacing(10)).padding(10),
        ].into()
    }
}
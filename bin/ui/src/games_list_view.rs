use super::App;

use crate::{
    Message, 
    OWNED_GAMES
};

use module::GameIdentifier;
use iced::font;
use iced::widget::{
    center_x, center_y, column, row, table, text, scrollable, button, checkbox, text_input
};
use iced::{Element, Font};
use std::collections::HashSet;
use std::cmp::Reverse;
use rayon::prelude::*;
use module::{
    Game,
    Module,
    game_targets::{
        get_game_targets,
        TargetStatus,
    },
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct GameListSettings {
    pub filter: GameListFilter,
    pub sort: GameListSort,
    pub has_achievements: bool,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum GameListFilter {
    Targets,
    #[default]
    InProgress,
    Completed,
    Perfected,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum GameListSort {
    #[default]
    ProgressDescending,
    ProgressAscending,
}

#[derive(Debug, Clone)]
pub struct GameListDisplay {
    //DISPLAY
    pub game_name: String,
    pub progress_display: String,
    //DATA
    pub id: GameIdentifier,
}

#[derive(Debug, Clone)]
pub struct GameListResult {
    pub settings: GameListSettings,
    pub list: Vec<GameListDisplay>,
}

impl GameListDisplay {
    pub async fn list(modules: Vec<Module>, list_settings: GameListSettings, title_search: Option<String>) -> GameListResult {
        let mut target_set: HashSet<GameIdentifier> = HashSet::new();
        let completion_map = module::get_modules_progress(&modules).expect("Failed to load completion status");
        for m in modules {
            get_game_targets(&m).expect("Failed to load targets")
                .iter()
                .filter(|t| matches!(t.status, TargetStatus::Target))
                .for_each(|t| {
                    target_set.insert(t.game_identifier.clone());
                });
        }

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
                match list_settings.filter {
                    GameListFilter::Targets => {
                        target_set.contains(&g.identifier)
                    }
                    GameListFilter::InProgress => {
                        !completion_map.get(&g.identifier).map(|c| c.complete).unwrap_or(false) || target_set.contains(&g.identifier)
                    },
                    GameListFilter::Completed => {
                        completion_map.get(&g.identifier).map(|c| c.complete).unwrap_or(false) && !target_set.contains(&g.identifier)
                    },
                    GameListFilter::Perfected => {
                        completion_map.get(&g.identifier).map(|c| c.perfect).unwrap_or(false) && !target_set.contains(&g.identifier)
                    }
                }
            })
            .filter(|g| {
                if list_settings.has_achievements {
                    completion_map.contains_key(&g.identifier)
                }
                else {
                    true
                }
            })
            .map(|g| (g, completion_map.get(&g.identifier).map(|p| p.progress.get_progress()).unwrap_or(0))) // Game, Progress
            .collect();
        match list_settings.sort {
            GameListSort::ProgressAscending => list.sort_by_key(|a| a.1),
            GameListSort::ProgressDescending => list.sort_by_key(|a| Reverse(a.1)),
        }
        

        GameListResult {
            settings: list_settings,
            list: list
                .par_iter()
                .map(|g| {
                    GameListDisplay{
                        game_name: g.0.name.clone(),
                        progress_display: g.1.to_string(),
                        id: g.0.identifier.clone(),
                    }
                })
                .collect()
        }
    }
}

impl App {
    pub fn game_list_view(&self, list_setting: GameListSettings) -> Element<'_, Message> {
        // Check if game list for selection has loaded
        let game_list = self.games.get(&list_setting);
        
        let filter_games = {
            row![
                button("Targets").on_press(Message::GamesView(GameListSettings { filter: GameListFilter::Targets, sort: list_setting.sort.clone(), has_achievements: list_setting.has_achievements })),
                button("In progress").on_press(Message::GamesView(GameListSettings { filter: GameListFilter::InProgress, sort: list_setting.sort.clone(), has_achievements: list_setting.has_achievements })),
                button("Completed").on_press(Message::GamesView(GameListSettings { filter: GameListFilter::Completed, sort: list_setting.sort.clone(), has_achievements: list_setting.has_achievements })),
                button("Perfected").on_press(Message::GamesView(GameListSettings { filter: GameListFilter::Perfected, sort: list_setting.sort.clone(), has_achievements: list_setting.has_achievements })),
            ]
        };

        let order_by = {
            row![
                button("Progress Asc").on_press(Message::GamesView(GameListSettings { filter: list_setting.filter.clone(), sort: GameListSort::ProgressAscending, has_achievements: list_setting.has_achievements })),
                button("Progress Dsc").on_press(Message::GamesView(GameListSettings { filter: list_setting.filter.clone(), sort: GameListSort::ProgressDescending, has_achievements: list_setting.has_achievements })),
            ]
        };

        let random_game = button("Random Game").on_press(Message::RandomGame);
        
        let achievement_filter = checkbox(list_setting.has_achievements)
            .label("Has Achievements")
            .on_toggle(move |b| Message::GamesView(GameListSettings { filter: list_setting.filter.clone(), sort: list_setting.sort.clone(), has_achievements: b }));

        
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
                    table::column(bold("Game Name"), |game: &GameListDisplay| button(game.game_name.as_str()).on_press(Message::GameView(game.id.clone()))),
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
            center_x(order_by).padding(5),
            center_x(achievement_filter).padding(5),
            center_x(random_game).padding(5),
            center_x(game_count).padding(5),
            center_x(game_search).padding(5),
            center_y(scrollable(center_x(main_view)).spacing(10)).padding(10),
        ].into()
    }
}
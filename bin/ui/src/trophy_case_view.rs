use super::App;

use crate::{Message, OWNED_GAMES};

use api::game_cover_fetch;
use iced::{Element};
use iced::widget::{
    column, row, text, image, image::Handle, grid, scrollable, center_x, button
};
use db::{
    game_completion_cache,
    game_target_store,
};
use std::collections::{HashMap, HashSet};
use rayon::prelude::*;

impl App {
    pub fn trophy_case_view(&self) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("Completed").on_press(Message::TrophyCaseView(TrophyCaseFilter::Completed)),
                button("Perfected").on_press(Message::TrophyCaseView(TrophyCaseFilter::Perfected)),
            ]
        };
        if let Some(trophies) = &self.trophies {
            let panes = trophies.iter().map(|app_id| {
                if let Some(i) =  self.game_covers.get(app_id) {
                    image(i).width(150).height(225).into()
                }
                else if let Some(game) = OWNED_GAMES.get(app_id) {
                    text(game.name.clone()).into()
                }
                else {
                    text("Loading").into()
                }
            });
            column![
                center_x(filter_games),
                scrollable(grid(panes).columns(10).spacing(10))
            ].into()
        }
        else {
            column![text("Loading trophy list...")].into()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TrophyCaseFilter {
    #[default]
    Completed,
    Perfected,
}

pub async fn load_trophies(view: TrophyCaseFilter) -> Vec<i32> {
    let target_set: HashSet<i32> = game_target_store::get_game_targets().expect("Failed to load targets")
        .iter()
        .filter(|t| !t.complete)
        .map(|t| t.app_id)
        .collect();
    game_completion_cache::get_game_completion()
        .expect("Failed to load cache")
        .iter()
        .filter(|c| {
            match view {
                TrophyCaseFilter::Completed => c.complete == 100 && !target_set.contains(&c.app_id),
                TrophyCaseFilter::Perfected => c.perfect && !target_set.contains(&c.app_id),
            }
        }) 
        .map(|c| c.app_id)
        .collect()
}

pub async fn load_game_covers(app_ids: Vec<i32>) -> HashMap<i32, Handle> {
    app_ids.par_iter()
        .map(|g| {
            (*g, game_cover_fetch::get_game_cover_blocking(g).map(Handle::from_bytes))
        })
        .filter(|t| t.1.is_some())
        .map(|t| (t.0, t.1.expect("All none will be filtered out")))
        .collect::<HashMap<_, _>>()
}
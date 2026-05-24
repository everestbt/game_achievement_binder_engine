use super::App;

use crate::{View, Message, OWNED_GAMES};

use api::game_cover_fetch;
use goals_lib::goals;
use iced::Element;
use iced::widget::{
    column, row, text, image, image::Handle, grid, scrollable, center_x, button, progress_bar
};
use std::collections::HashMap;
use rayon::prelude::*;


impl App {
    pub fn trophy_case_view(&self) -> Element<'_, Message> {
        let filter_games = {
            row![
                button("Completed").on_press(Message::TrophyCaseView(TrophyCaseFilter::Completed)),
                button("Perfected").on_press(Message::TrophyCaseView(TrophyCaseFilter::Perfected)),
            ]
        };

        let filter = match &self.view {
            View::TrophyCase(filter) => filter,
            _ => unreachable!("Should only be call when in trophy case view")
        };

        if let Some(trophies) = self.trophies.get(&filter) {
            let game_progress = progress_bar(0.0..=OWNED_GAMES.len() as f32, trophies.len() as f32);

            // Grid of game trophies
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
                center_x(game_progress),
                scrollable(grid(panes).columns(10).spacing(10))
            ].into()
        }
        else {
            column![text("Loading trophy list...")].into()
        }
    }
}

#[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub enum TrophyCaseFilter {
    #[default]
    Completed,
    Perfected,
}

pub async fn load_trophies(view: TrophyCaseFilter) -> (TrophyCaseFilter, Vec<i32>) {
    (view.clone(), goals::get_game_completion()
        .iter()
        .filter(|c| {
            match view {
                TrophyCaseFilter::Completed => c.1.complete,
                TrophyCaseFilter::Perfected => c.1.perfect,
            }
        }) 
        .map(|c| *c.0)
        .collect())
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
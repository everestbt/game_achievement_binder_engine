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
            let game_progress: Element<'_, Message> = {
                let percent_progress = 100.0 * trophies.len() as f32 / OWNED_GAMES.len() as f32;
                column![
                    center_x(text(format!("Game progress: {done}/{total} ({percent:.2}%)", done = trophies.len(), total = OWNED_GAMES.len(), percent = percent_progress))),
                    center_x(progress_bar(0.0..=OWNED_GAMES.len() as f32, trophies.len() as f32)),
                ].into()
            };
            let achievement_progress: Element<'_, Message> = if let Some(progress) = &self.achievement_progress {
                match filter {
                    TrophyCaseFilter::Completed => { 
                        let unlocked_and_excluded = (progress.unlocked_achievements + progress.total_excluded) as f32;
                        let percent_progress = 100.0 * unlocked_and_excluded / progress.total_achievements as f32;
                        column![
                            center_x(text(format!("Achievement progress: {done}/{total} ({percent:.2}%)", done = unlocked_and_excluded, total = progress.total_achievements, percent = percent_progress))),
                            center_x(progress_bar(0.0..=progress.total_achievements as f32, unlocked_and_excluded)),
                        ].into()
                    },
                    TrophyCaseFilter::Perfected => {
                        let percent_progress = 100.0 * (progress.unlocked_achievements) as f32 / progress.total_achievements as f32;
                        column![
                            center_x(text(format!("Achievement progress: {done}/{total} ({percent:.2}%)", done = progress.unlocked_achievements, total =  progress.total_achievements, percent = percent_progress))),
                            center_x(progress_bar(0.0..=progress.total_achievements as f32, (progress.unlocked_achievements) as f32)),
                        ].into()
                    },
                }
            }
            else {
                text("Loading achievement progress").into()
            };

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
                center_x(achievement_progress),
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

#[derive(Debug, Clone)]
pub struct TotalAchievementProgress {
    pub total_achievements : u32,
    pub unlocked_achievements: u32,
    pub total_excluded: u32,
}

pub async fn load_achievement_progress() -> TotalAchievementProgress {
    let mut total_achievements = 0;
    let mut unlocked_achievements = 0;
    let mut total_excluded = 0;
    for g in goals::get_game_progress() {
        total_achievements += g.1.total;
        unlocked_achievements += g.1.unlocked;
        total_excluded += g.1.excluded;
    }
    TotalAchievementProgress {
        total_achievements,
        unlocked_achievements,
        total_excluded
    }
}

pub async fn load_game_covers(app_ids: Vec<i32>) -> HashMap<i32, Handle> {
    app_ids.par_iter()
        .map(|g| {
            (*g, game_cover_fetch::get_game_cover_blocking(g).map(Handle::from_bytes))
        })
        .filter(|t| t.1.is_ok())
        .map(|t| (t.0, t.1.expect("All none will be filtered out")))
        .collect::<HashMap<_, _>>()
}
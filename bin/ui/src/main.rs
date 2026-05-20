mod games_list_view;
mod goals_view;
mod game_view;
mod trophy_case_view;

use iced::widget::{
    center_x, column, row, button, image::Handle, text, 
};
use iced::{Element, Theme, Task};
use games_list_view::{
    GameListDisplay, 
    GameListFilter, 
    GameListResult
};
use goals_view::Goal;
use api::game_fetch::{self, Game};
use simple_error::SimpleError;
use std::env;
use std::collections::HashMap;
use std::sync::LazyLock;
use db::{
    steam_id_store,
    game_target_store,
    excluded_achievement_store,
};
use goals_lib::goals;
use game_view::{GameDisplay, GameGoalDisplay};
use api::achievement_fetch::GameAchievement;
use trophy_case_view::TrophyCaseFilter;

// We only need to load this once, do it statically so it can be shared between all threads
pub static OWNED_GAMES: LazyLock<HashMap<i32, Game>> = LazyLock::new(|| {
        let key = env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key");
        let steam_id = steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first");
        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        // Sync and update all data
        runtime.block_on(goals::get_and_sync_completed_achievements(&key, &steam_id));
        let owned_games_vec = runtime.block_on(game_fetch::get_owned_games(&key, &steam_id));
        owned_games_vec.iter().map(|g| (g.appid, g.clone())).collect::<HashMap<_, _>>()
    }
);

pub fn main() -> iced::Result {
    // Do this call to instantiate the owned games list before the program starts
    OWNED_GAMES.len();
    color_eyre::install().expect("Failed to install color eyre");
    iced::application(App::new, App::update, App::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    GamesView(GameListFilter),
    GameView(i32), //app_id
    GameLoaded(GameDisplay),
    GoalIconsLoaded(HashMap<(i32, String), Handle>), // app_id, achievement_name -> Image
    GoalsView,
    GoalsLoaded(Vec<Goal>),
    AchievementCheckboxToggled(bool),
    GamesLoaded(GameListResult),
    GenerateRandomAchievement(i32), // app_id
    RandomAchievementGenerated(Result<(Game, Option<GameAchievement>), SimpleError>), 
    SetAsGameTarget(i32), // app_id
    SetGameAsComplete(i32), // app_id
    RandomGame,
    ExcludeAchievement(i32, String), // app_id, achievement_name
    TrophyCaseView(TrophyCaseFilter),
    TrophiesLoaded(Vec<i32>), // app_id's
    GameCoversLoaded(HashMap<i32, Handle>), // app_id -> Game Cover
    CachesSynced(Result<(), SimpleError>),
    GameListSearch(String),
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    None,
    Goals,
    Games(GameListFilter),
    Game(i32), // app_id
    TrophyCase,
}

#[derive(Debug, Clone)]
struct Credentials {
    key: String,
    steam_id: String,
}

struct App {
    // SETTINGS
    view: View,
    // DISPLAY
    games: HashMap<(GameListFilter, bool), Vec<GameListDisplay>>, // filter, has_achievement -> game_list
    games_have_achievements_filter: bool,
    game_list_search: String,
    goals: Option<Vec<Goal>>,
    game_views: HashMap<i32, GameDisplay>,
    goal_icons: HashMap<(i32, String), Handle>, // app_id, achievement_name -> image
    trophies: Option<Vec<i32>>,
    game_covers: HashMap<i32, Handle>, // app_id -> image
    // DATA
    credentials: Credentials,
}

impl App {
    fn new() -> Self {
        let credentials = load_credentials();
        tokio::runtime::Runtime::new().expect("Unable to create a runtime").block_on(sync_caches(credentials.clone())).expect("Failed to sync caches");
        Self {
            view: View::default(),
            games: HashMap::new(),
            games_have_achievements_filter: true,
            game_list_search: "".to_string(),
            goals: None,
            game_views: HashMap::new(),
            goal_icons: HashMap::new(),
            game_covers: HashMap::new(),
            trophies: None,
            credentials,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GamesView(filter) => {
                self.view = View::Games(filter.clone());
                Task::perform(GameListDisplay::list(self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
            },
            Message::GamesLoaded(list_result) => {
                self.games.insert((list_result.filter, list_result.has_achievements), list_result.list);
                Task::none()
            },
            Message::GameView(id) => {
                self.view = View::Game(id);
                Task::perform(game_view::load_game_display(self.credentials.clone(), id, OWNED_GAMES.get(&id).expect("Does not exist").name.clone()), Message::GameLoaded)
            },
            Message::GameLoaded(display) => {
                let filtered_icons: Vec<GameGoalDisplay> = display.goals.iter()
                    .filter(|i| !self.goal_icons.contains_key(&(display.app_id, i.achievement_name.clone()))).cloned()
                    .collect();
                let task = if filtered_icons.is_empty() {
                    Task::none()
                } 
                else {
                    Task::perform(game_view::load_all_goal_icons(display.app_id, filtered_icons), Message::GoalIconsLoaded)
                };
                self.game_views.insert(display.app_id, display);
                task
            },
            Message::GoalIconsLoaded(icons) => {
                for icon in icons {
                    self.goal_icons.insert(icon.0, icon.1);
                }
                Task::none()
            },
            Message::GoalsView => {
                self.view = View::Goals;
                if self.goals.is_none() {
                    Task::perform(Goal::list(self.credentials.clone()), Message::GoalsLoaded)
                }
                else {
                    Task::none()
                }
            },
            Message::GoalsLoaded(goals) => {
                let mut tasks: Vec<Task<Message>> = Vec::new();
                for g in &goals {
                    tasks.push(Task::perform(game_view::load_game_display(self.credentials.clone(), g.app_id, g.game_name.clone()), Message::GameLoaded));
                }
                self.goals = Some(goals);
                Task::batch(tasks)
            },
            Message::AchievementCheckboxToggled(is_checked) => {
                self.games_have_achievements_filter = is_checked;
                match &self.view {
                    View::Games(filter) => {
                        Task::perform(GameListDisplay::list(self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
                    },
                    _ => Task::none()
                }
            },
            Message::GenerateRandomAchievement(ref app_id) => Task::perform(game_view::generate_random_achievement(self.credentials.clone(), *app_id), Message::RandomAchievementGenerated),
            Message::RandomAchievementGenerated(random_achievement) => {
                if let Ok(r) = random_achievement {
                    let tasks = vec![
                        Task::perform(Goal::list(self.credentials.clone()), Message::GoalsLoaded), 
                        Task::perform(game_view::load_game_display(self.credentials.clone(), r.0.appid, r.0.name.clone()), Message::GameLoaded)
                    ];
                    self.handle_generated_random_achievement(r.0, r.1);
                    Task::batch(tasks)
                }
                else {
                    panic!("{}", random_achievement.unwrap_err().as_str())
                }
            },
            Message::SetAsGameTarget(app_id) => {
                game_target_store::save_game_target(&app_id, &false).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.target = true;
                }
                Task::perform(sync_caches(self.credentials.clone()), Message::CachesSynced)
            },
            Message::SetGameAsComplete(app_id) => {
                game_target_store::save_game_target(&app_id, &true).expect("Failed to save target");
                if let Some(view) = self.game_views.get_mut(&app_id) {
                    view.complete = true;
                }
                Task::perform(sync_caches(self.credentials.clone()), Message::CachesSynced)
            },
            Message::RandomGame => {
                let random_game_id = OWNED_GAMES.values().nth(rand::random_range(..OWNED_GAMES.values().len())).unwrap().appid;
                self.view = View::Game(random_game_id).clone();
                Task::perform(game_view::load_game_display(self.credentials.clone(), random_game_id, OWNED_GAMES.get(&random_game_id).expect("Does not exist").name.clone()), Message::GameLoaded)
            },
            Message::ExcludeAchievement(app_id, achievement_name) => {
                excluded_achievement_store::save_excluded_achievement(&achievement_name, &app_id).expect("Failed to exclude achievement");
                let tasks = vec![
                    Task::perform(game_view::load_game_display(self.credentials.clone(), app_id, OWNED_GAMES.get(&app_id).expect("Does not exist").name.clone()), Message::GameLoaded),
                    Task::perform(sync_caches(self.credentials.clone()), Message::CachesSynced)
                ];
                Task::batch(tasks)
            },
            Message::TrophyCaseView(filter) => {
                self.view = View::TrophyCase;
                Task::perform(trophy_case_view::load_trophies(filter), Message::TrophiesLoaded)
            },
            Message::TrophiesLoaded(trophies) => {
                let filtered_covers: Vec<i32> = trophies.iter()
                    .filter(|app_id| !self.game_covers.contains_key(app_id)).copied()
                    .collect();
                self.trophies = Some(trophies);
                if filtered_covers.is_empty() {
                    Task::none()
                }
                else {
                    Task::perform(trophy_case_view::load_game_covers(filtered_covers), Message::GameCoversLoaded)
                }
            },
            Message::GameCoversLoaded(cover_map) => {
                for cover in cover_map {
                    self.game_covers.insert(cover.0, cover.1);
                }
                Task::none()
            },
            Message::CachesSynced(result) => {
                if result.is_err() {
                    panic!("Caches failed to sync")
                }
                let mut tasks: Vec<Task<Message>> = vec![];
                for k in self.games.keys() {
                    tasks.push(Task::perform(GameListDisplay::list(k.1, k.0.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded));
                }
                self.trophies = None;
                Task::batch(tasks)
            },
            Message::GameListSearch(search) => {
                self.game_list_search = search.clone();
                
                match &self.view {
                    View::Games(filter) => {
                        Task::perform(GameListDisplay::list(self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
                    },
                    _ => Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let view_selector = {
            row![
                button("Games").on_press(Message::GamesView(GameListFilter::default())),
                button("Goals").on_press(Message::GoalsView),
                button("Trophy Case").on_press(Message::TrophyCaseView(TrophyCaseFilter::default())),
            ]
        };

        let main_view: Element<'_, Message> = match &self.view {
            View::None => column![center_x(text("Welcome to G.A.B.E"))].into(),
            View::Goals => self.goal_view(),
            View::Games(filter) => self.game_list_view(filter.clone()),
            View::Game(_) => self.game_view(),
            View::TrophyCase => self.trophy_case_view(),
        };

        column![
            center_x(view_selector).padding(10),
            main_view,
        ]
        .into()
    }
}

fn load_credentials() -> Credentials {
    Credentials { 
        key: env::var("STEAM_API_KEY").expect("You need to set the environment variable STEAM_API_KEY with your API key"),
        steam_id: steam_id_store::get_id().expect("Failed to load steam-id, use the cli and supply a --id first")
    }
}

async fn sync_caches(credentials: Credentials) -> Result<(), SimpleError> {
    goals::get_and_sync_completed_achievements(&credentials.key, &credentials.steam_id).await;
    let owned_games = OWNED_GAMES.values().cloned().collect();
    goals::refresh_game_completion_cache(&credentials.key, &credentials.steam_id, &owned_games).await;
    Ok(())
}

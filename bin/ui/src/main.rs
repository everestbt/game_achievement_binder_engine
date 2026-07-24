mod games_list_view;
mod goals_view;
mod game_view;
mod trophy_case_view;

use iced::widget::{
    center_x, column, row, button, image::Handle, text, 
};
use iced::window::Settings;
use iced::{Element, Theme, Task};
use games_list_view::{
    GameListDisplay, 
    GameListFilter, 
    GameListResult
};
use goals_view::Goal;
use simple_error::{SimpleResult};
use std::collections::HashMap;
use std::sync::LazyLock;
use steam_db::{
    game_target_store,
    excluded_achievement_store,
};
use steam_utils::goals;
use game_view::{GameDisplay, GameGoalDisplay};
use trophy_case_view::{
    TrophyCaseFilter, 
    TotalAchievementProgress
};
use module::{
    Game, 
    GameAchievement,
    game_cover::save_game_cover,
    Module,
};
use anyhow::{
    Result, 
    anyhow,
};

// We only need to load this once, do it statically so it can be shared between all threads
pub static OWNED_GAMES: LazyLock<HashMap<i32, Game>> = LazyLock::new(|| {
        let modules = module::get_modules().expect("Failed to load modules");
        let mut owned_return : HashMap<i32, Game> = HashMap::new();
        let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        for m in modules {
            match m {
                module::Module::STEAM(key, steam_id) => {
                    // Sync and update all data
                    runtime.block_on(goals::sync_caches(&key, &steam_id));
                    let owned_games_vec = runtime.block_on(module::get_module_games(module::Module::STEAM(key, steam_id)));
                    owned_games_vec.iter().for_each(|g| {owned_return.insert(g.id.clone(), g.clone());});
                },
                _ => todo!()
            }
        }
        owned_return
    }
);

pub fn main() -> iced::Result {
    // Do this call to instantiate the owned games list before the program starts
    OWNED_GAMES.len();
    color_eyre::install().expect("Failed to install color eyre");
    let window_settings = Settings{ maximized: true, ..Settings::default() };
    iced::application(App::new, App::update, App::view)
        .window(window_settings)
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
    RandomAchievementGenerated(SimpleResult<(Game, Option<GameAchievement>)>), 
    SetAsGameTarget(i32), // app_id
    SetGameAsComplete(i32), // app_id
    RandomGame,
    ExcludeAchievement(i32, String), // app_id, achievement_name
    TrophyCaseView(TrophyCaseFilter),
    TrophiesLoaded((TrophyCaseFilter, Vec<i32>)), // app_id's
    AchievementProgressLoaded(TotalAchievementProgress),
    GameCoversLoaded(HashMap<i32, Handle>), // app_id -> Game Cover
    CachesSynced(SimpleResult<()>),
    GameListSearch(String),
    EditGameCover(i32), // app_id
    GameCoverURLInput(String),
    SaveGameCover,
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    None,
    Goals,
    Games(GameListFilter),
    Game(i32), // app_id
    TrophyCase(TrophyCaseFilter),
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
    trophies: HashMap<TrophyCaseFilter, Vec<i32>>,
    achievement_progress: Option<TotalAchievementProgress>,
    game_covers: HashMap<i32, Handle>, // app_id -> image
    // DATA
    credentials: Credentials,
}

impl App {
    fn new() -> Self {
        let credentials = load_credentials().expect("Failed to load credentials");
        tokio::runtime::Runtime::new().expect("Unable to create a runtime").block_on(sync_caches(credentials.clone())).expect("Failed to sync caches");
        Self {
            view: View::default(),
            games: HashMap::new(),
            games_have_achievements_filter: false,
            game_list_search: "".to_string(),
            goals: None,
            game_views: HashMap::new(),
            goal_icons: HashMap::new(),
            game_covers: HashMap::new(),
            trophies: HashMap::new(),
            achievement_progress: None,
            credentials,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GamesView(filter) => {
                self.view = View::Games(filter.clone());
                Task::perform(GameListDisplay::list(self.credentials.clone(), self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
            },
            Message::GamesLoaded(list_result) => {
                self.games.insert((list_result.filter, list_result.has_achievements), list_result.list);
                Task::none()
            },
            Message::GameView(id) => {
                self.view = View::Game(id);
                let mut tasks = vec![
                    Task::perform(game_view::load_game_display(self.credentials.clone(), id, OWNED_GAMES.get(&id).expect("Does not exist").name.clone()), Message::GameLoaded),
                ];
                if !self.game_covers.contains_key(&id) {
                    tasks.push(Task::perform(trophy_case_view::load_game_covers(vec![id]), Message::GameCoversLoaded))
                }
                Task::batch(tasks)
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
                    Task::perform(Goal::list(), Message::GoalsLoaded)
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
                        Task::perform(GameListDisplay::list(self.credentials.clone(), self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
                    },
                    _ => Task::none()
                }
            },
            Message::GenerateRandomAchievement(ref app_id) => Task::perform(game_view::generate_random_achievement(self.credentials.clone(), *app_id), Message::RandomAchievementGenerated),
            Message::RandomAchievementGenerated(random_achievement) => {
                if let Ok(r) = random_achievement {
                    let tasks = vec![
                        Task::perform(Goal::list(), Message::GoalsLoaded), 
                        Task::perform(game_view::load_game_display(self.credentials.clone(), r.0.id, r.0.name.clone()), Message::GameLoaded)
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
                let random_game_id = OWNED_GAMES.values().nth(rand::random_range(..OWNED_GAMES.values().len())).unwrap().id;
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
                self.view = View::TrophyCase(filter.clone());
                let tasks = vec![
                    Task::perform(trophy_case_view::load_trophies(filter), Message::TrophiesLoaded),
                    Task::perform(trophy_case_view::load_achievement_progress(), Message::AchievementProgressLoaded)
                ];
                Task::batch(tasks)
            },
            Message::TrophiesLoaded((filter, trophies)) => {
                let filtered_covers: Vec<i32> = trophies.iter()
                    .filter(|app_id| !self.game_covers.contains_key(app_id)).copied()
                    .collect();
                self.trophies.insert(filter, trophies);
                if filtered_covers.is_empty() {
                    Task::none()
                }
                else {
                    Task::perform(trophy_case_view::load_game_covers(filtered_covers), Message::GameCoversLoaded)
                }
            },
            Message::AchievementProgressLoaded(progress) => {
                self.achievement_progress = Some(progress);
                Task::none()
            }
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
                    tasks.push(Task::perform(GameListDisplay::list(self.credentials.clone(), k.1, k.0.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded));
                }
                self.trophies = HashMap::new();
                Task::batch(tasks)
            },
            Message::GameListSearch(search) => {
                self.game_list_search = search.clone();
                
                match &self.view {
                    View::Games(filter) => {
                        Task::perform(GameListDisplay::list(self.credentials.clone(), self.games_have_achievements_filter, filter.clone(), Some(self.game_list_search.clone())), Message::GamesLoaded)
                    },
                    _ => Task::none()
                }
            },
            Message::EditGameCover(app_id) => {
                if let Some(g) = self.game_views.get_mut(&app_id) {
                    g.game_cover_edit = true;
                }
                Task::none()
            },
            Message::GameCoverURLInput(input) => {
                // This should only be done from a game view, where we can then get the app_id
                match self.view {
                    View::Game(app_id) => {
                        if let Some(game) = self.game_views.get_mut(&app_id) {
                            game.game_cover_url = input;
                            Task::none()
                        }
                        else {
                            unreachable!("Should not be possible to call this when game view is not loaded")
                        }
                    },
                    _ => unreachable!("Should not be called from another view")
                }
            },
            Message::SaveGameCover => {
                // This should only be done from a game view, where we can then get the app_id
                match self.view {
                    View::Game(app_id) => {
                        if let Some(game) = self.game_views.get_mut(&app_id) {
                            println!("Setting url {}", game.game_cover_url);
                            save_game_cover(Module::STEAM(self.credentials.key.clone(), self.credentials.steam_id.clone()), &app_id, &game.game_cover_url, ).expect("Failed to save game cover");
                            game.game_cover_edit = false;
                            Task::perform(trophy_case_view::load_game_covers(vec![app_id]), Message::GameCoversLoaded)
                        }
                        else {
                            unreachable!("Should not be possible to call this when game view is not loaded")
                        }
                    },
                    _ => unreachable!("Should not be called from another view")
                }
            }
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
            View::TrophyCase(_) => self.trophy_case_view(),
        };

        column![
            center_x(view_selector).padding(10),
            main_view,
        ]
        .into()
    }
}

fn load_credentials() -> Result<Credentials> {
    let modules = module::get_modules()?;
    for m in modules {
        match m {
            module::Module::STEAM(key, steam_id) => {
                return Ok(Credentials { 
                    key: key.clone(),
                    steam_id: steam_id.clone(),
                })
            },
            _ => {}
        }
    }
    Err(anyhow!("Did not find steam credentials, run cli with a steam id first"))
}

async fn sync_caches(credentials: Credentials) -> SimpleResult<()> {
    goals::sync_caches(&credentials.key, &credentials.steam_id).await;
    Ok(())
}

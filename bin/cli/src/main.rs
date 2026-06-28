use steam_api::{achievement_fetch::{GameAchievement}, game_fetch};
use steam_db::{steam_id_store, achievement_store, excluded_achievement_store, request_store, game_completion_cache};
use goals_lib::{goals};

use std::{collections::HashMap, env, io};
use clap::Parser;

// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Your Steam ID
    #[arg(long)]
    id: Option<String>,

    /// Return a random achievement
    #[arg(long)]
    random_achievement: bool,

    /// Return a random game with achievements remaining on it
    #[arg(long)]
    random_game: bool,

    /// Return a list of the current goals
    #[arg(long)]
    goals: bool,

    /// Exclude an achievement by its id in the goals list
    #[arg(long)]
    exclude_achievement: Option<i32>,

    /// Return a list of completed games
    #[arg(long)]
    completed_games: bool,

    /// Return a list of games that are closest to being done
    #[arg(long)]
    game_completion_list: bool,

    /// Game name used to filter goals
    #[arg(long)]
    game_name: Option<String>,

    /// Purge specific data tables
    #[arg(long)]
    purge: Option<String>,

    /// Show debug level information
    #[arg(short, long)]
    debug: bool,
}

struct Credentials {
    key: String,
    steam_id: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    if args.random_achievement {
        let credentials = get_credentials(&args);
        let game = request_game_name(&credentials.key, &credentials.steam_id).await.expect("No game found for search");

        let random_achievement: Option<GameAchievement> = goals::get_random_achievement_for_game(&credentials.key, &credentials.steam_id, &game.appid).await;
        if let Some(a) = random_achievement {
            println!("And your selected achievement is:");
            println!(
                "{achievement} : {description}",
                achievement = a.display_name,
                description = a
                    .description
                    .clone()
                    .unwrap_or("no description".to_string())
                );
            // Save the achievement
            achievement_store::save_achievement(&a.name, &a.display_name, &a.description, &game.appid, &game.last_played).expect("Failed to save achievement");
            println!("Saved the achievement!");
        }
        else {
            println!("No achievements left in this game!");
        }
    }
    else if args.random_game {
        // Fetch games
        let credentials = get_credentials(&args);
        let mut owned_games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await;
        let mut game_and_achievement: Option<(game_fetch::Game, GameAchievement)> = None;
        while !owned_games.is_empty() {
            let index = (rand::random::<f32>() * owned_games.len() as f32).floor() as usize;
            let random_game = owned_games.remove(index);
            let random_achievement: Option<GameAchievement> = goals::get_random_achievement_for_game(&credentials.key, &credentials.steam_id, &random_game.appid).await;
            if let Some(a) = random_achievement {
                game_and_achievement = Some((random_game, a));
                break;
            }
        }
        match game_and_achievement {
            Some(g_a) => {
                // Show the results
                println!("Found the game {:#?}!", g_a.0.name);
                println!("And your selected achievement is:");
                println!(
                    "{achievement} : {description}",
                    achievement = g_a.1.display_name,
                    description = g_a.1
                        .description
                        .clone()
                        .unwrap_or("no description".to_string())
                        );
                
                // Save the achievement
                achievement_store::save_achievement(&g_a.1.name, &g_a.1.display_name, &g_a.1.description, &g_a.0.appid, &g_a.0.last_played).expect("Failed to save achievement");
                println!("Saved the achievement!");
            },
            None => println!("No games left with any achievements")
        }
    }
    else if args.goals {
        let credentials = get_credentials(&args);
        let owned_games: HashMap<i32, game_fetch::Game> = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await.iter().map(|n| (n.appid, n.clone())).collect();
        // Refresh caches and get goals
        goals::sync_caches(&credentials.key, &credentials.steam_id).await;
        let mut achievements: Vec<achievement_store::Achievement> = achievement_store::get_achievements().expect("Failed to load achievements");
        achievements.sort_by(|a, b| i32::cmp(&a.app_id,&b.app_id));
        
        for a in achievements {
            if a.description.is_none() {
                println!("{game} : {name} [{id}]", name = a.display_name, game = owned_games.get(&a.app_id).expect("Achievement goal for unowned game?!?").name, id = a.id);
            }
            else{
                println!("{game} : {name} - {description} [{id}]", name = a.display_name, game = owned_games.get(&a.app_id).expect("Achievement goal for unowned game?!?").name, description = a.description.clone().unwrap(), id = a.id);
            }
        }
    }
    else if args.exclude_achievement.is_some() {
        let achievement = achievement_store::get_achievement(&args.exclude_achievement.unwrap()).expect("Achievement no found");
        // First delete the achievement, if this is all that succeeds then it is at least off the list
        achievement_store::delete_achievement(&args.exclude_achievement.unwrap()).expect("Failed to delete achievement");
        // Add it to the list of excluded achievements
        excluded_achievement_store::save_excluded_achievement(&achievement.achievement_name, &achievement.app_id).expect("Failed to save the exclusion");
    }
    else if args.completed_games {
        // Get full game list
        let credentials = get_credentials(&args);
        let games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await;
        goals::sync_caches(&credentials.key, &credentials.steam_id).await;
        let completed_games = goals::get_game_completion();
        for g in completed_games {
            if g.1.complete {
                let game = games.iter().find(|game| game.appid == g.0).unwrap();
                println!("Completed game: {name}", name = game.name);
            }
        }
    }
    else if args.game_completion_list {
        // Get full game list
        let credentials = get_credentials(&args);
        let games: Vec<game_fetch::Game> = game_fetch::get_owned_games(&credentials.key, &credentials.steam_id).await;
        goals::sync_caches(&credentials.key, &credentials.steam_id).await;
        let progressed_games = goals::get_game_completion();
        let progress = goals::get_game_progress();
        for g in progressed_games {
            if !g.1.complete {
                let game = games.iter().find(|game| game.appid == g.0).unwrap();
                println!("{name} : {progress}", name = game.name, progress = progress.get(&g.0).map(|p| p.get_progress()).unwrap_or(0));
            }
        }
    }
    else if args.purge.is_some()
        && args.purge.is_some_and(|f| f == "completed_games") {
            game_completion_cache::drop_table().expect("Failed to drop table");
        }

    if args.debug {
        let request_count = request_store::get_count().expect("Failed to load request count");
        println!("Request count {count}", count = request_count);
    }
    Ok(())
}

fn get_credentials(args: &Args) -> Credentials {
    let key_var = env::var("STEAM_API_KEY");
    if key_var.is_err() {
        panic!("You need to set the environment variable STEAM_API_KEY with your API key")
    }
    let key = key_var.unwrap();

    let steam_id= if let Some(id) = &args.id {
        steam_id_store::save_id(id).expect("Failed to save the id");
        println!("Saved your steam id, no need to use --id each time now. You can replace it by using --id again.");
        id.clone()
    }
    else {
        steam_id_store::get_id().expect("Failed to load a key, use --id first")
    };

    Credentials { key, steam_id }
}

async fn request_game_name(key : &str, steam_id : &str) -> Option<game_fetch::Game> {
    let mut game_name= String::new();
    println!("Please enter the game name:");  
  
    io::stdin()  
        .read_line(&mut game_name) 
        .expect("Failed to read line");
	
    // Fetch games and search for it
    let game_name_lowercase: String = game_name.trim().to_lowercase();
    let game_list: Vec<game_fetch::Game> = game_fetch::get_owned_games(key, steam_id).await
        .iter()
        .filter(|a| a.name.to_lowercase().contains(&game_name_lowercase))
        .cloned()
        .collect();

    if game_list.is_empty() {
        println!("Failed to find that game");
        Option::None
    } 
    else if game_list.len() > 1 {
        println!("There are a few games to choose from:");
        let mut i = 0;
        for ele in game_list.iter() {
            println!("{game_name} [{num}]", game_name = ele.name, num = i);
            i += 1;
        }
        println!("Enter the number of which one you would like to use");
        let mut choice= String::new();
        io::stdin()  
            .read_line(&mut choice) 
            .expect("Failed to read line");
        let index: i32 = choice.trim().parse().expect("It needs to be an integer");
        if index < 0 || index >= i {
            panic!("Enter a value that is included in the index")
        }
        Option::Some(game_list.get(index as usize).unwrap().clone())
    }
    else {
        Option::Some(game_list.first().unwrap().clone())
    }
}

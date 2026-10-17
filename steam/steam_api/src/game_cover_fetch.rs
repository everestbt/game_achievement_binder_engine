use bytes::Bytes;
use steam_db::game_cover_store;
use anyhow::Result;

pub async fn get_game_cover(app_id: &i32) -> Result<Bytes> {
    let url = if let Some(cover) = game_cover_store::get_game_cover(app_id).expect("Failed to load game cover database").map(|g| g.url) {
        cover
    }
    else {
        "https://shared.steamstatic.com/store_item_assets/steam/apps/".to_owned() + &app_id.to_string() + "/library_600x900_2x.jpg"
    };
    let req = reqwest::get(url).await.and_then(|r| r.error_for_status()).and_then(|r| Ok(r.bytes()));
    Ok(req?.await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[tokio::test]
    async fn test_load_game_cover() {
        // Use an app_id which we know has a game cover
        let app_id = 1794680;
        assert!(get_game_cover(&app_id).await.is_ok())
    }

    #[test]
    fn test_load_game_cover_not_present() {
        // Use test database incase a local database is present
        temp_env::with_var("GABE_DB_PREFIX", Some("test"), || {
            // Use an app_id which we know doesn't have a stored game cover
            let app_id = 4035270;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .name("test_load_game_cover_not_present")
                .build()
                .unwrap()
                .block_on(async {
                    assert!(get_game_cover(&app_id).await.is_err())
                })
        });
    }
}
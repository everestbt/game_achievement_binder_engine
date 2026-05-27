use bytes::Bytes;
use simple_error::{SimpleResult, SimpleError};

pub fn get_game_cover_blocking(app_id: &i32) -> SimpleResult<Bytes> {
    let url = "https://shared.steamstatic.com/store_item_assets/steam/apps/".to_owned() + &app_id.to_string() + "/library_600x900_2x.jpg";
    let result = reqwest::blocking::get(url).and_then(|r| r.error_for_status()).and_then(|r| r.bytes());
    match result {
        Ok(r) => {
            Ok(r)
        }
        Err(e) => {
            Err(SimpleError::with("Failed to load game cover url", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_game_cover() {
        // Use an app_id which we know has a game cover
        let app_id = 1794680;
        assert!(get_game_cover_blocking(&app_id).is_ok())
    }

    #[test]
    fn test_load_game_cover_not_present() {
        // Use an app_id which we know doesn't have a stored game cover
        let app_id = 4035270;
        assert!(get_game_cover_blocking(&app_id).is_err())
    }
}
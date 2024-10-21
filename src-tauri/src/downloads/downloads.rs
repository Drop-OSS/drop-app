/* GENERAL OUTLINE
When downloading any game, the following details must be provided to the server:
 - Game ID
 - User token
 - TBC

The steps to then download a game are as follows:
 1. User requests  
 */
use tauri::AppHandle;
use crate::auth::generate_authorization_header;
use crate::DB;
use crate::db::DatabaseImpls;

#[tauri::command]
fn download_game(app: AppHandle, game_id: String) -> Result<String, String>{
    todo!()
}




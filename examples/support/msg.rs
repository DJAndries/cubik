use serde::{Serialize, Deserialize};
use cubik::player::PlayerControlMessage;

#[derive(Serialize, Deserialize)]
pub enum AppMessage {
	Chat { text: String, sender: Option<u8> },
	PlayerChange { msg: PlayerControlMessage }
}

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum AppMessage {
	Chat { text: String, sender: Option<u8> }
}

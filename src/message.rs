use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Debug)]
pub enum CommMessage<M> {
	PlayerChange {
		player_id: u8,
		joined: bool
	},
	PlayerNameStatement {
		name: String
	},
	WhoYouAre {
		player_id: u8
	},
	App(M)
}

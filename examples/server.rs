use cubik::server::ServerContainer;
use serde::{Serialize, Deserialize};

const PORT: u16 = 27020;

#[derive(Serialize, Deserialize)]
pub enum AppMessage {
	A(usize)
}

fn main() {
	let mut server_container: ServerContainer<AppMessage> = ServerContainer::new(PORT, 10).unwrap();

	println!("server listening on port {}", PORT);

	loop {
		server_container.update();
	}
}

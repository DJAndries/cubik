mod support;

use cubik::server::ServerContainer;
use cubik::player::{Player, PlayerControlType};
use cubik::quadoctree::{QuadOctreeNode, BoundingBox};
use support::msg::AppMessage;
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::collections::HashMap;
use crate::support::constants::APP_ID;
use cubik::map::GameMap;

const PORT: u16 = 27020;

fn main() {
	let mut server_container: ServerContainer<AppMessage> = ServerContainer::new(PORT, 10).unwrap();

	println!("server listening on port {}", PORT);

	let mut last_status_update = Instant::now();
	let mut player_map: HashMap<u8, Player> = HashMap::new();

	let map = GameMap::load_map("models/map2", APP_ID, None, None, true).unwrap();

	let mut last_time = Instant::now();

	loop {
		server_container.update();

		let current_pids = server_container.pids();
		player_map.retain(|&k, _| current_pids.contains(&k));

		for pid in current_pids {
			let player = player_map.entry(pid)
				.or_insert(Player::new([0., 1.5, 0.], PlayerControlType::MultiplayerServer, [-0.28, 0.275, 0.0], [0.44, 0.275, 0.08]));
			if let Ok(msgs) = server_container.get_msgs(pid) {
				for msg in msgs {
					if let AppMessage::PlayerChange { msg, .. } = msg {
						player.update(0., Some(map.quadoctree.as_ref().unwrap()), None, Some(msg));
					}
				}
				
			}
			if let Some(msg) = player.update(last_time.elapsed().as_secs_f32(), Some(map.quadoctree.as_ref().unwrap()), None, None) {
				server_container.broadcast(AppMessage::PlayerChange {
					msg: msg,
					player_id: pid
				});
			}
		}

		if last_status_update.elapsed().as_secs_f32() > 5. {
			last_status_update = Instant::now();
			println!("peer status update:");
			for (pid, conn) in &server_container.connections {
				println!("pid: {} name: {}", pid, conn.name.as_ref().unwrap_or(&"".to_string()));
			}
			println!("");
			server_container.broadcast(AppMessage::Chat {
				text: format!("I see {} peers", server_container.connections.len()),
				sender: None
			});
		}

		last_time = Instant::now();

		sleep(Duration::from_millis(17));
	}
}

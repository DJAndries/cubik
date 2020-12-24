mod support;

use cubik::server::ServerContainer;
use cubik::player::{Player, PlayerControlType};
use cubik::quadoctree::{QuadOctreeNode, BoundingBox};
use support::msg::AppMessage;
use std::time::Instant;
use std::thread::sleep_ms;
use std::collections::HashMap;
use crate::support::constants::APP_ID;

const PORT: u16 = 27020;

fn main() {
	let mut server_container: ServerContainer<AppMessage> = ServerContainer::new(PORT, 10).unwrap();

	println!("server listening on port {}", PORT);

	let mut last_status_update = Instant::now();
	let mut player_map: HashMap<u8, Player> = HashMap::new();

	let mut quadoctree = QuadOctreeNode::new_tree(BoundingBox {
		start_pos: [-25., -25., -25.],
		end_pos: [25., 25., 25.]
	}, false);

	let map_obj = cubik::wavefront::load_obj("models/map2.obj", APP_ID, None, None,
		&[1., 1., 1.], Some(&mut quadoctree), None).unwrap();

	let mut last_time = Instant::now();

	loop {
		server_container.update();

		let current_pids = server_container.pids();
		player_map.retain(|&k, _| current_pids.contains(&k));

		for pid in current_pids {
			let mut player = player_map.entry(pid)
				.or_insert(Player::new([0., 1.5, 0.], PlayerControlType::MultiplayerServer, [-0.28, 0.275, 0.0], [0.44, 0.275, 0.08]));
			if let Ok(msgs) = server_container.get_msgs(pid) {
				for msg in msgs {
					if let AppMessage::PlayerChange { msg, .. } = msg {
						player.update(0., Some(&quadoctree), None, Some(msg));
					}
				}
				server_container.broadcast(AppMessage::PlayerChange {
					msg: player.update(last_time.elapsed().as_secs_f32(), Some(&quadoctree), None, None).unwrap(),
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

		sleep_ms(17);
	}
}

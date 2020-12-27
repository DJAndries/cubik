mod support;

use cubik::server::ServerContainer;
use support::msg::AppMessage;
use std::time::{Instant, Duration};
use std::thread::sleep;

const PORT: u16 = 27020;

fn main() {
	let mut server_container: ServerContainer<AppMessage> = ServerContainer::new(PORT, 10).unwrap();

	println!("server listening on port {}", PORT);

	let mut last_status_update = Instant::now();

	loop {
		server_container.update();

		for pid in server_container.pids() {
			if let Ok(msgs) = server_container.get_msgs(pid) {
				for msg in msgs {
					server_container.broadcast(msg);
				}
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

		sleep(Duration::from_millis(16));
	}
}

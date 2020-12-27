mod support;

use cubik::client::ClientContainer;
use support::msg::AppMessage;
use std::io::{self, Write};
use std::collections::HashSet;
use std::time::{Instant, Duration};
use std::thread::sleep;

const PORT: u16 = 27020;

fn player_name(ctr: &ClientContainer<AppMessage>, pid: Option<u8>) -> String {
	match pid {
		Some(pid) => {
			ctr.peers.get(&pid).unwrap().name.as_ref().unwrap_or(&format!("Player {}", pid)).clone()
		},
		None => {
			"Server".to_string()
		}
	}
}

fn display_player_changes(ctr: &ClientContainer<AppMessage>, recognized_peers: HashSet<(u8, String)>) -> HashSet<(u8, String)> {
	let current_pids = ctr.peers.iter().filter_map(|(pid, meta)| {
		if let Some(name) = &meta.name {
			return Some((*pid, name.clone()));
		}
		None
	}).collect();
	for (_, name) in recognized_peers.difference(&current_pids) {
		println!("** {} left **", name);
	}
	for (_, name) in current_pids.difference(&recognized_peers) {
		println!("** {} joined **", name);
	}
	current_pids
}

fn main() {
	print!("Enter your name: ");
	io::stdout().flush().unwrap();
	let mut username = String::new();
	io::stdin().read_line(&mut username).unwrap();
	
	username = username.trim().to_string();

	let mut client_container: ClientContainer<AppMessage> = ClientContainer::new(format!("127.0.0.1:{}", PORT).as_str()).unwrap();
	let mut recognized_peers: HashSet<(u8, String)> = HashSet::new();
	client_container.state_name(username).unwrap();

	let mut curr_count = 0;
	let mut last_count_inc_time = Instant::now();

	loop {
		client_container.update().unwrap();

		recognized_peers = display_player_changes(&client_container, recognized_peers);

		for msg in client_container.get_msgs() {
			if let AppMessage::Chat { text, sender } = msg {
				println!("{} says: {}", player_name(&client_container, sender), text);
			}
		}

		if last_count_inc_time.elapsed().as_secs_f32() > 1. {
			last_count_inc_time = Instant::now();
			curr_count += 1;
			client_container.send(AppMessage::Chat {
				text: format!("This is message {}", curr_count),
				sender: client_container.player_id
			}).unwrap();
		}

		sleep(Duration::from_millis(16));
	}
}

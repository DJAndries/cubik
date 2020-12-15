use std::io;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use derive_more::{From, Error, Display};
use serde::{Serialize, de::DeserializeOwned};
use crate::message::CommMessage;
use crate::message;

#[derive(From, Error, Debug, Display)]
pub enum ServerError {
	IOError(io::Error),
	MessageError(message::MessageError),
	PlayerNotFound
}

pub struct ServerConn<M: Serialize + DeserializeOwned> {
	pub stream: TcpStream,
	buffer: Vec<u8>,
	pub incoming_msgs: Vec<M>,
	pub name: Option<String>
}

pub struct ServerContainer<M: Serialize + DeserializeOwned> {
	pub listener: TcpListener,
	pub connections: HashMap<u8, ServerConn<M>>,
	pub next_player_id: u8,
	pub max_players: usize
}

impl<M: Serialize + DeserializeOwned> ServerContainer<M> {
	pub fn new(port: u16, max_players: usize) -> Result<Self, ServerError> {
		let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))?;
		listener.set_nonblocking(true)?;
		Ok(Self {
			listener: listener,
			connections: HashMap::new(), 
			next_player_id: 1,
			max_players: max_players
		})
	}

	pub fn pids(&self) -> Vec<u8> {
		self.connections.keys().cloned().collect()
	}

	pub fn update(&mut self) {
		while let Ok((stream, _)) = self.listener.accept() {
			if self.connections.len() < self.max_players {
				if let Ok(()) = stream.set_nonblocking(true) {
					self.player_joined(stream);
				}
			}
		}

		for pid in self.pids() {
			self.receive_from(pid);
		}
	}

	fn player_joined(&mut self, stream: TcpStream) {
		let pid = self.next_player_id;
		self.connections.insert(pid, ServerConn::<M> {
			stream: stream,
			buffer: Vec::new(),
			incoming_msgs: Vec::new(),
			name: None
		});
		self.next_player_id += 1;
		self.broadcast(&CommMessage::PlayerChange {
			player_id: pid,
			joined: true
		});
		self.send_to(pid, &CommMessage::WhoYouAre { player_id: pid });
	}

	fn player_leave(&mut self, player_id: u8) {
		self.connections.remove(&player_id);
		self.broadcast(&CommMessage::PlayerChange {
			player_id: player_id,
			joined: false
		});
	}

	pub fn get_msgs(&mut self, player_id: u8) -> Result<Vec<M>, ServerError> {
		let conn = self.connections.get_mut(&player_id).ok_or(ServerError::PlayerNotFound)?;

		let mut result: Vec<M> = Vec::new();
		result.append(&mut conn.incoming_msgs);
		Ok(result)
	}

	pub fn broadcast(&mut self, message: &CommMessage<M>) {
		for pid in self.pids() {
			self.send_to(pid, message);
		}
	}

	fn receive_from(&mut self, player_id: u8) -> Result<(), ServerError> {
		let conn = self.connections.get_mut(&player_id).ok_or(ServerError::PlayerNotFound)?;

		match message::receive(&mut conn.stream, &mut conn.buffer) {
			Err(e) => {
				self.player_leave(player_id);
				return Err(ServerError::from(e));
			},
			Ok(msg) => {
				if let Some(msg) = msg {
					self.process_msg(player_id, msg);
				}
			}
		};

		Ok(())
	}

	fn process_msg(&mut self, pid: u8, msg: CommMessage<M>) {
		let conn = self.connections.get_mut(&pid).unwrap();

		match msg {
			CommMessage::PlayerNameStatement { player_id, name } => {
				if player_id == pid {
					conn.name = Some(name.clone());
					self.broadcast(&CommMessage::PlayerNameStatement {
						player_id: player_id,
						name: name
					});
				}
			},
			CommMessage::App(msg) => conn.incoming_msgs.push(msg),
			_ => ()
		};
	}

	pub fn send_to(&mut self, player_id: u8, message: &CommMessage<M>) -> Result<(), ServerError> {
		let conn = self.connections.get_mut(&player_id).ok_or(ServerError::PlayerNotFound)?;

		if let Err(e) = message::send(&mut conn.stream, message) {
			self.player_leave(player_id);
			return Err(ServerError::from(e));
		}

		return Ok(())
	}
}

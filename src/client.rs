use std::io;
use std::str::FromStr;
use std::io::{Read, Write};
use std::time::Duration;
use crate::message::CommMessage;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream, AddrParseError};
use serde::{Serialize, de::DeserializeOwned};
use crate::message;
use derive_more::{From, Error, Display};

#[derive(From, Error, Debug, Display)]
pub enum ClientError {
	IOError(io::Error),
	MessageError(message::MessageError),
	AddrParseError(AddrParseError)
}

#[derive(Default)]
pub struct PeerMeta {
	name: Option<String>
}

pub struct ClientContainer<M: Serialize + DeserializeOwned> {
	pub stream: TcpStream,
	pub peers: HashMap<u8, PeerMeta>,
	pub incoming_msgs: Vec<M>,
	pub player_id: Option<u8>,
	buffer: Vec<u8>
}

impl<M: Serialize + DeserializeOwned> ClientContainer<M> {
	pub fn new(addr: &str) -> Result<Self, ClientError> {
		let addr = SocketAddr::from_str(addr)?;
		let stream = TcpStream::connect_timeout(&addr, Duration::new(5, 0))?;

		Ok(Self {
			stream: stream,
			peers: HashMap::new(),
			incoming_msgs: Vec::new(),
			buffer: Vec::new(),
			player_id: None
		})
	}

	pub fn send(&mut self, message: &CommMessage<M>) -> Result<(), ClientError> {
		Ok(message::send(&mut self.stream, message)?)
	}

	pub fn update(&mut self) -> Result<(), ClientError> {
		if let Some(msg) = message::receive(&mut self.stream, &mut self.buffer)? {
			self.process_msg(msg);
		}

		Ok(())
	}

	fn process_msg(&mut self, msg: CommMessage<M>) {
		match msg {
			CommMessage::WhoYouAre { player_id } => self.player_id = Some(player_id),
			CommMessage::PlayerChange { player_id, joined } => {
				if joined {
					self.peers.insert(player_id, Default::default());
				} else {
					self.peers.remove(&player_id);
				}
			},
			CommMessage::PlayerNameStatement { player_id, name } => {
				if let Some(peer) = self.peers.get_mut(&player_id) {
					peer.name = Some(name);
				}
			},
			CommMessage::App(msg) => self.incoming_msgs.push(msg),
			_ => ()
		};
	}

	pub fn get_msgs(&mut self, player_id: u8) -> Vec<M> {
		let mut result: Vec<M> = Vec::new();
		result.append(&mut self.incoming_msgs);
		result
	}
}

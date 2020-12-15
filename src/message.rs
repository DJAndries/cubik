use std::io;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::net::TcpStream;
use derive_more::{From, Error, Display};

#[derive(Serialize, Deserialize, Debug)]
pub enum CommMessage<M> {
	PlayerChange {
		player_id: u8,
		joined: bool
	},
	PlayerNameStatement {
		player_id: u8,
		name: String
	},
	WhoYouAre {
		player_id: u8
	},
	App(M)
}

#[derive(From, Error, Debug, Display)]
pub enum MessageError {
	IOError(io::Error),
	SerializeError(bincode::Error)
}

pub fn send<M: Serialize + DeserializeOwned>(stream: &mut TcpStream, message: &CommMessage<M>) -> Result<(), MessageError> {
	let serialized: Vec<u8> = bincode::serialize(message)?;
	let mut send_buf: Vec<u8> = Vec::with_capacity(serialized.len() + 4);
	send_buf.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
	send_buf.extend_from_slice(&serialized);

	stream.write_all(&send_buf)?;

	Ok(())
}

pub fn receive<M: Serialize + DeserializeOwned>(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<Option<CommMessage<M>>, MessageError> {
	stream.read_to_end(buffer)?;

	if buffer.len() < 5 {
		return Ok(None);
	}

	let mut le_bytes = [0u8; 4];
	le_bytes.copy_from_slice(&buffer[..4]);
	let msg_size = u32::from_le_bytes(le_bytes) as usize;
	if buffer.len() - 4 < msg_size as usize {
		return Ok(None);
	}

	let decoded = bincode::deserialize::<CommMessage<M>>(&buffer[4..(msg_size + 4)])?;

	buffer.drain(..(msg_size + 4));

	Ok(Some(decoded))
}

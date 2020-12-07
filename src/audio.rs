use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::io;
use rodio::{Source, Sink, decoder::{LoopedDecoder, Decoder, DecoderError}, OutputStream, OutputStreamHandle, StreamError, PlayError};
use derive_more::{Display, From, Error};

pub type SoundData = Vec<u8>;
pub type SoundStream = (OutputStream, OutputStreamHandle);

#[derive(Display, Debug, From, Error)]
pub enum AudioError {
	IOError(io::Error),
	RodioDecoderError(DecoderError),
	RodioStreamError(StreamError),
	RodioPlayError(PlayError),
}

pub fn buffer_sound(filename: &str) -> Result<SoundData, AudioError> {
	let mut file = File::open(filename)?;
	let mut data = Vec::new();
	file.read_to_end(&mut data)?;
	Ok(data)
}

pub fn get_sound_stream() -> Result<SoundStream, AudioError> {
	Ok(OutputStream::try_default()?)
}

pub fn sound_decoder_from_data(data: &SoundData) -> Result<Decoder<Cursor<Vec<u8>>>, AudioError> {
	let cursor = Cursor::new(data.clone());
	Ok(Decoder::new(cursor)?)
}

pub fn sound_decoder_from_file(filename: &str) -> Result<Decoder<BufReader<File>>, AudioError> {
	let file = File::open(filename)?;
	let reader = BufReader::new(file);

	Ok(Decoder::new(reader)?)
}

pub fn sound_decoder_from_data_looped(data: &SoundData) -> Result<LoopedDecoder<Cursor<Vec<u8>>>, AudioError> {
	let cursor = Cursor::new(data.clone());
	Ok(Decoder::new_looped(cursor)?)
}

pub fn sound_decoder_from_file_looped(filename: &str) -> Result<LoopedDecoder<BufReader<File>>, AudioError> {
	let file = File::open(filename)?;
	let reader = BufReader::new(file);
	Ok(Decoder::new_looped(reader)?)
}

pub fn play_sound_from_data(stream: &SoundStream, data: &SoundData) -> Result<(), AudioError> {
	let cursor = Cursor::new(data.clone());
	let decoder = Decoder::new(cursor)?;
	stream.1.play_raw(decoder.convert_samples());
	Ok(())
}

pub fn play_sound_from_file(stream: &SoundStream, filename: &str) -> Result<(), AudioError> {
	let file = File::open(filename)?;
	let reader = BufReader::new(file);
	
	let decoder = Decoder::new(reader)?;
	stream.1.play_raw(decoder.convert_samples());
	Ok(())
}

pub fn create_sink(stream: &SoundStream) -> Result<Sink, AudioError> {
	Ok(Sink::try_new(&stream.1)?)
}

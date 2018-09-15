extern crate clap;
extern crate enigo;
extern crate midir;

use std::error::Error;
use std::time::Duration;
use std::thread;
use std::fmt::Write;

use clap::{App, Arg};

use enigo::*;

use midir::{Ignore, MidiInput, MidiInputConnection};

/// The amount of time to wait for a keydown event to stick
const KEY_DELAY_MS: u64 = 40;

#[derive(Debug, PartialEq)]
enum MidiEvent {
	NoteOn,
	NoteOff,
}

#[derive(Debug)]
struct MidiMessage {
	event: MidiEvent,
	channel: u8,
	note: u8,
	velocity: u8,
}

#[derive(Debug)]
enum MidiError {
	TooShort,
	Unimplemented(u8),
}

fn parse_message(message: &[u8]) -> Result<MidiMessage, MidiError> {
	match message[0] & 0xf0 {
		0x80 => if message.len() < 3 {
			Err(MidiError::TooShort)
		} else {
			Ok(MidiMessage {
				event: MidiEvent::NoteOff,
				channel: message[0] & 0x0f,
				note: message[1] & 0x7f,
				velocity: message[2] & 0x7f,
			})
		},
		0x90 => if message.len() < 3 {
			Err(MidiError::TooShort)
		} else {
			let velocity = message[2] & 0x7f;
			let event = if velocity != 0 {
				MidiEvent::NoteOn
			} else {
				MidiEvent::NoteOff
			};
			Ok(MidiMessage {
				event: event,
				channel: message[0] & 0x0f,
				note: message[1] & 0x7f,
				velocity: velocity,
			})
		},
		0xB0 => if message.len() < 3 {
			Err(MidiError::TooShort)
		} else {
			Ok(MidiMessage {
				event: MidiEvent::NoteOff,
				channel: message[0] & 0x0f,
				note: message[1] & 0x7f,
				velocity: message[2] & 0x7f,
			})
		},

		_ => Err(MidiError::Unimplemented(message[0])),
	}
}

fn main() {
	let matches = App::new("Midi Perform")
		.version("0.2.9")
		.author("Sean Cross <sean@xobs.io>")
		.about("Accepts MIDI controller data and simulates keyboard presses")
		.arg(
			Arg::with_name("list")
				.short("l")
				.long("list")
				.help("List available devices"),
		)
		.arg(
			Arg::with_name("device")
				.short("d")
				.long("device")
				.help("Connect to specified device")
				.value_name("DEVICE"),
		)
		.get_matches();

	if matches.is_present("list") {
		list_devices().expect("unable to list MIDI devices");
		return;
	}

	let device_name = match matches.value_of("device") {
		Some(s) => Some(s.to_owned()),
		None => None,
	};
	run(device_name).unwrap();
}

fn midi_callback(_timestamp_us: u64, raw_message: &[u8], keygen: &mut enigo::Enigo) {
	let keys = vec![
		'z', '`', 'x', '1', 'c', 'v', '2', 'b','3', 'n', '4', 'm', 'a','5','s', '6', 'd', 'f', '7', 'g','8','h','8','j','q','0','w','-','e','r','=','t',']','y','\\','u','i'
	];

	if let Ok(msg) = parse_message(raw_message) {
		//println!("Parsed Message: {:?}", msg);

		if msg.note > 72 {
			println!("Note too high (max: C-6 [72]) Got {}", msg.note);
			return;
		} else if msg.note < 36 {
			println!("Note too low (min: C-3 [36]) Got {}", msg.note);
			return;
		}

		let note_idx = (msg.note - 36) as usize;
		
		if msg.event == MidiEvent::NoteOn {
			println!("Sending key: {}", keys[note_idx]);
			keygen.key_down(enigo::Key::Layout(keys[note_idx]));
			thread::sleep(Duration::from_millis(KEY_DELAY_MS));
			keygen.key_up(enigo::Key::Layout(keys[note_idx]));
			return;
		} else if msg.event == MidiEvent::NoteOff {
			return;
		}	
	
	}

	/*let mut s = String::new();
	for &byte in raw_message {
		write!(&mut s, "{:X} ", byte).expect("Unable to write");
	}
	println!("Unhandled message for data: {}", s);*/
}

fn run(midi_name: Option<String>) -> Result<(), Box<Error>> {
	let mut target_device_name = midi_name.to_owned();

	let mut device_idx: Option<usize> = None;

	let mut connection: Option<MidiInputConnection<()>> = None;

	loop {
		let mut midi_in = MidiInput::new("keyboard-tweak")?;
		midi_in.ignore(Ignore::None);

		// If the index of the device has changed, reset the connection
		if let Some(idx) = device_idx {
			match midi_in.port_name(idx) {
				Err(_) => {
					device_idx = None;
					connection = None;
				}
				Ok(val) => {
					if let Some(ref name) = target_device_name {
						if &val != name {
							device_idx = None;
							connection = None;
						}
					}
				},
			}
		} else {
			device_idx = None;
			connection = None;
		};

		// If there is no connection, try to create a new one.
		if connection.is_none() {
			match target_device_name {
				None => println!("Connecting to first available device"),
				Some(ref s) => println!("Looking for device {}", s),
			}

			for i in 0..midi_in.port_count() {
				match midi_in.port_name(i) {
					Err(_) => (),
					Ok(name) => {
						match target_device_name {
							Some(ref s) => 
								if &name == s {
									println!("Using device: {}", i);
									device_idx = Some(i);
								},
							None => {
								println!("Using device: {}", i);
								device_idx = Some(i);
								target_device_name = Some(name);
							},
						}
					}
				}
				println!("    {}", midi_in.port_name(i)?);
			}
		}

		if connection.is_none() {
			if let Some(idx) = device_idx {
				let mut keygen = enigo::Enigo::new();
				match midi_in.connect(
					idx,
					"key monitor",
					move |ts, raw_msg, _ignored| {
						midi_callback(ts, raw_msg, &mut keygen);
					},
					(),
				) {
					Err(reason) => println!("Unable to connect to device: {:?}", reason),
					Ok(conn) => {
						println!("Connection established");
						connection = Some(conn);
					}
				}
			}
		}
		thread::sleep(Duration::from_secs(1));
	}
}

fn list_devices() -> Result<(), Box<Error>> {
	let mut midi_in = MidiInput::new("keyboard-tweak")?;
	midi_in.ignore(Ignore::None);

	println!("Available MIDI devices:");
	for i in 0..midi_in.port_count() {
		println!("    {}", midi_in.port_name(i)?);
	}

	Ok(())
}
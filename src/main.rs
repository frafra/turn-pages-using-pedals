extern crate midir;
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, Key};

use std::io::{stdin, stdout, Write};
use std::error::Error;

use midir::{MidiInput, Ignore};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid input port selected")?
        }
    };
    
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let mut keys = AttributeSet::<Key>::new();
    keys.insert(Key::KEY_DOWN);
    keys.insert(Key::KEY_UP);
    let mut device = VirtualDeviceBuilder::new()?
        .name("pedals-remapped")
        .with_keys(&keys)?
        .build()
        .unwrap();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        match message {
            [176, 66, 127] => {
                device.emit(&[InputEvent::new(EventType::KEY, Key::KEY_DOWN.code(), 1)]).unwrap();
                device.emit(&[InputEvent::new(EventType::KEY, Key::KEY_DOWN.code(), 0)]).unwrap();
            },
            [176, 67, 127] => {
                device.emit(&[InputEvent::new(EventType::KEY, Key::KEY_UP.code(), 1)]).unwrap();
                device.emit(&[InputEvent::new(EventType::KEY, Key::KEY_UP.code(), 0)]).unwrap();
            },
            _ => {},
        }
    }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

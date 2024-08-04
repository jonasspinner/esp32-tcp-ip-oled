use shared::{Command, Serialize};
use std::io::Write;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("192.168.178.101:8080")?;

    Command::FillRandom.serialize(&mut stream)?;

    loop {
        Command::StepGOL.serialize(&mut stream)?;
        sleep(Duration::from_millis(500));
        stream.flush()?;
    }
}

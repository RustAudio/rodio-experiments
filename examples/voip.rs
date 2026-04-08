use clap::Parser;
use rodio_experiments::ConstSource;
use std::error::Error;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use rodio::cpal::Sample;
use rodio::microphone::MicrophoneBuilder;

#[derive(clap::Parser)]
enum Cli {
    Play {
        /// format: <IPv4>:<port>
        address: String,
    },
    Serve {
        port: u16,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Cli::parse();

    match options {
        Cli::Serve { port } => {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || recorder(tx));
            listen(port, rx)?;
        }
        Cli::Play { address } => {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || player(rx));
            connect_and_read(address, tx)?;
        }
    };

    Ok(())
}

pub fn connect_and_read(address: String, tx: mpsc::Sender<Vec<u16>>) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(&address).expect("always a valid address");
    socket.connect(address)?;
    socket.send(&[0xde, 0xce])?;

    let mut buf = [0u8; 4000 * size_of::<u16>()];
    loop {
        let Ok(len) = socket.recv(&mut buf) else {
            continue;
        };
        assert_eq!(len, buf.len(), "can not deal with partial buffers");

        let buf = buf
            .chunks_exact(2)
            .map(|a| u16::from_le_bytes([a[0], a[1]]))
            .collect::<Vec<_>>();
        tx.send(buf)?;
    }
}

struct MpscSource {
    buffered: std::vec::IntoIter<u16>,
    rx: mpsc::Receiver<Vec<u16>>,
}

impl ConstSource<44100, 1> for MpscSource {
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for MpscSource {
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(res) = self.buffered.next() {
            return Some(res.to_sample());
        }

        let buffer = self.rx.recv().ok()?;
        self.buffered = buffer.into_iter();
        self.buffered.next().map(|s| s.to_sample())
    }
}

fn player(rx: mpsc::Receiver<Vec<u16>>) {
    let source = MpscSource {
        rx,
        buffered: Vec::new().into_iter(),
    };

    let stream_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
    let player = rodio::Player::connect_new(stream_handle.mixer());
    player.append(source.into_fixed_source());
    player.sleep_until_end();
}

pub fn listen(port: u16, rx: mpsc::Receiver<Vec<u16>>) -> Result<(), Box<dyn Error>> {
    let host = UdpSocket::bind(("0.0.0.0", port))?;
    let mut buf = [0u8; 4000 * size_of::<u16>()];

    loop {
        if let Ok((len, addr)) = host.recv_from(&mut buf)
            && len == 2
            && buf[0] == 0xde
            && buf[1] == 0xce
        {
            let msg = rx.recv().expect("microphone keeps going forever");
            let msg =
                unsafe { core::slice::from_raw_parts(msg.as_ptr() as *const u8, msg.len() * 2) };
            if let Err(e) = host.send_to(&msg, &addr) {
                eprintln!("could not send audio packet to host: {e}");
                continue;
            }
        }
    }
}

fn recorder(tx: Sender<Vec<u16>>) {
    let stream = MicrophoneBuilder::new()
        .default_device()
        .unwrap()
        .default_config()
        .unwrap()
        .open_stream()
        .unwrap();

    let mut buffer = Vec::with_capacity(4096);
    for sample in stream {
        let sample: u16 = sample.to_sample();

        if buffer.len() < 4096 {
            buffer.push(sample);
            continue;
        } else {
            tx.send(std::mem::take(&mut buffer))
                .expect("listener should not end");
            buffer.reserve(4096);
        }
    }
}

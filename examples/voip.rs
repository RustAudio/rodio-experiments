use clap::Parser;
use rodio::nz;
use rodio::speakers::SpeakersBuilder;
use rodio_experiments::ConstSource;
use rodio_experiments::dynamic_source_ext::ExtendDynamicSource;
use rodio_experiments::fixed_source::FixedSourceExt;
use std::error::Error;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use dasp_sample::Sample;
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
    match Cli::parse() {
        Cli::Serve { port } => stream(port),
        Cli::Play { address } => connect_and_read(address),
    }
}

pub fn connect_and_read(address: String) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(&address)?;
    socket.connect(address)?;
    socket.send(&[0xde, 0xce])?;

    let source = SocketSource {
        socket,
        next: usize::MAX,
        buffered: Vec::with_capacity(4096),
    };

    SpeakersBuilder::new()
        .default_device()?
        .default_config()?
        .prefer_channel_counts([nz!(1)])
        .prefer_channel_counts([nz!(44100)])
        .play(source.into_fixed_source())?;
    thread::sleep(Duration::from_secs(10));
    Ok(())
}

struct SocketSource {
    buffered: Vec<f32>,
    next: usize,
    socket: UdpSocket,
}

impl ConstSource<44100, 1> for SocketSource {
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for SocketSource {
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(res) = self.buffered.get(self.next) {
            self.next += 1;
            return Some(*res);
        }

        let mut buf = [0u8; 4096 * size_of::<u16>()];
        let len = self.socket.recv(&mut buf).unwrap();
        assert_eq!(len, buf.len(), "can not deal with partial buffers");

        self.buffered.clear();
        self.buffered.extend(
            buf.chunks_exact(2)
                .map(|a| u16::from_le_bytes([a[0], a[1]]))
                .map(|s| s.to_sample::<rodio::Sample>()),
        );

        self.next = 1;
        Some(self.buffered[0])
    }
}

fn stream(port: u16) -> Result<(), Box<dyn Error>> {
    let host = UdpSocket::bind(("0.0.0.0", port))?;

    let mut stream = MicrophoneBuilder::new()
        .default_device()?
        .default_config()?
        .prefer_sample_rates([nz!(44100)])
        .prefer_channel_counts([nz!(1)])
        .open_stream()?
        .try_as_fixed_source()?
        .with_channel_count(nz!(1))
        .with_sample_rate(nz!(44100))
        .try_into_const_source::<44100, 1>()?;

    let mut samples = Vec::new();
    let mut bytes = Vec::new();
    loop {
        let mut buf = [0u8, 2];
        let Ok((_, addr)) = host.recv_from(&mut buf) else {
            continue;
        };

        if buf != [0xde, 0xce] {
            continue;
        };

        for sample in &mut stream {
            let sample: u16 = sample.to_sample();
            if samples.len() < 4096 {
                samples.push(sample);
            } else {
                bytes.clear();
                bytes.extend(samples.drain(..).flat_map(|sample| sample.to_le_bytes()));
                if let Err(e) = host.send_to(&bytes, addr) {
                    eprintln!("could not send audio packet to host: {e}");
                    continue;
                }
            }
        }
    }
}

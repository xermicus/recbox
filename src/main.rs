//! `recbox` connects to jack and records 2 output ports into a wave file.
//!
//! This depends on (exactly) 2 output ports being available, usually
//! "system:capture_{1,2}"), which will be connected automatically.
//!
//! The output dir is provided by the `RECBOX_OUT_DIR` env var and defaults
//! to `/tmp`. Filename is `<unixseconds>.wav`. The file is flushed once per
//! second.
//!
//! Intended usage is to use this as a deamon via systemd, to start recording
//! on boot (see also the provided system unit files).

#![allow(clippy::declare_interior_mutable_const)]
#![allow(clippy::borrow_interior_mutable_const)]

use std::{
    cell::LazyCell,
    sync::{Arc, Mutex},
};

use hound::{WavSpec, WavWriter};
use jack::{
    contrib::ClosureProcessHandler, AsyncClient, AudioIn, Client, ClientOptions, ClientStatus,
    Control, NotificationHandler, PortFlags, ProcessHandler, ProcessScope,
};

const JACK_CLIENT_NAME: &str = "recbox";
const JACK_PORT_L: &str = "in_l";
const JACK_PORT_R: &str = "in_r";
const AMPLITUDE: f32 = i16::MAX as f32;
const OUT_DIR: LazyCell<String> =
    LazyCell::new(|| std::env::var("RECBOX_OUT_DIR").unwrap_or_else(|_| String::from("/tmp")));

const fn wav_spec() -> WavSpec {
    WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    }
}

fn connect_jack() -> Result<Client, jack::Error> {
    let (client, status) = Client::new(JACK_CLIENT_NAME, ClientOptions::NO_START_SERVER)?;
    assert_eq!(
        status,
        ClientStatus::empty(),
        "jack client opened but returned a status error"
    );
    Ok(client)
}

fn connect_ports<N, P>(
    client: &mut AsyncClient<N, P>,
    out_ports: &[String; 2],
) -> Result<(), jack::Error>
where
    N: 'static + Send + Sync + NotificationHandler,
    P: 'static + Send + ProcessHandler,
{
    let src = &out_ports[0];
    let dst = &format!("{JACK_CLIENT_NAME}:{JACK_PORT_L}");
    log::info!("connecting {src} with {dst}");
    client.as_client().connect_ports_by_name(src, dst)?;

    let src = &out_ports[1];
    let dst = &format!("{JACK_CLIENT_NAME}:{JACK_PORT_R}");
    log::info!("connecting {src} with {dst}");
    client.as_client().connect_ports_by_name(src, dst)?;

    Ok(())
}

fn out_file() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("should not travel in time")
        .as_secs();
    format!("{}/{}.wav", OUT_DIR.clone().as_str(), now)
}

fn main() {
    env_logger::init();

    let client = connect_jack().unwrap();
    log::info!("{JACK_CLIENT_NAME} connected to jack");

    let in_l = client
        .register_port(JACK_PORT_L, AudioIn::default())
        .unwrap();
    let in_r = client
        .register_port(JACK_PORT_R, AudioIn::default())
        .unwrap();
    let out_ports = client.ports(None, None, PortFlags::IS_OUTPUT);
    let out_file = out_file();

    log::info!("ports IN: {:?}", [in_l.name(), in_r.name()]);
    log::info!("ports OUT: {:?}", &out_ports);
    log::info!("out file: {out_file}");
    log::info!("WAV spec: {:?}", wav_spec());

    let writer = Arc::new(Mutex::new(
        WavWriter::create(out_file.as_str(), wav_spec()).unwrap(),
    ));
    let w = Arc::clone(&writer);
    let callback = ClosureProcessHandler::new(move |_: &Client, ps: &ProcessScope| -> Control {
        let in_l = in_l.as_slice(ps);
        let in_r = in_r.as_slice(ps);
        let mut writer = w.lock().unwrap();
        for (sample_l, sample_r) in in_l.iter().zip(in_r.iter()) {
            writer.write_sample((sample_l * AMPLITUDE) as i16).unwrap();
            writer.write_sample((sample_r * AMPLITUDE) as i16).unwrap();
        }
        Control::Continue
    });
    let mut client = client.activate_async((), callback).unwrap();
    log::info!("client active");

    connect_ports(&mut client, &out_ports.try_into().unwrap()).unwrap();
    log::info!("ports connected");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        writer.lock().unwrap().flush().unwrap();
    }
}

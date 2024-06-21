mod managers;
mod packages;
mod temperature;
mod threadpool;

use crate::packages::packet_dispatcher::dispatch_packet;
use rand::Rng;
use shared::packages::connack::Connack;
use shared::packages::connect::Connect;
use shared::packages::packet::FixedHeader;
use shared::packages::packet::PacketType;
use shared::packages::packet::ReadablePacket;
use shared::packages::packet::WritablePacket;
use shared::packages::suback::Suback;
use shared::packages::subscribe::Subscribe;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use temperature::conversion_utils::build_list;
use temperature::temperature_entry::TemperatureEntry;

use crate::managers::measuresmanager::MeasuresManager;

static TIME_CHECK_NEW_MEASURES: u64 = 1000;
static SERVER_URL: &str = "localhost:3090";
static TIME_OUT: u64 = 100;
static POOL_SIZE: usize = 4;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = threadpool::ThreadPool::new(POOL_SIZE).unwrap();

    let mut socket =
        TcpStream::connect(SERVER_URL.to_string()).expect("Failed to connect to server");
    let _ = connect_to_broker(&mut socket).expect("Failed to connect to MQTT broker");
    let _ = subscribe_to_topic(&mut socket).expect("Failed to subscribe to topic");

    let measures_manager_arc_mutex = Arc::new(Mutex::new(MeasuresManager::new()));
    let measures_manager = Arc::clone(&measures_manager_arc_mutex);

    start_broker_listener(socket.try_clone().unwrap(), measures_manager.clone());

    for stream in listener.incoming() {
        let new_measures_manager = measures_manager.clone();
        pool.execute(move || {
            let http_stream = stream.unwrap();
            handle_http_connection(http_stream, new_measures_manager);
        });
    }
}

fn connect_to_broker(socket: &mut TcpStream) -> Result<(), String> {
    let connect = Connect {
        client_id: "http_server".to_string(),
        username: "http_server".to_string(),
        password: "1234".to_string(),
        last_will_topic: "temperature/status".to_string(),
        last_will_message: "temperature publisher stopped working".to_string(),
        keep_alive: 60,
        last_will_qos: 0,
        clean_session: 1,
        last_will_retain: 0,
        last_will_flag: 0_u8,
    };

    match connect.write_to(socket) {
        Ok(_) => {
            let fixed_header = FixedHeader::read_fixed_header(socket).unwrap();

            match PacketType::from_u8(fixed_header.packet_type) {
                Some(PacketType::Connack) => {
                    let connack = Connack::read_from(socket, fixed_header).unwrap();
                    println!("Received: {:?}", connack);
                    Ok(())
                }
                _ => Err("Unexpected package header (Expected Connack)".to_string()),
            }
        }
        Err(_) => Err("Failed to write package to socket".to_string()),
    }
}

fn subscribe_to_topic(socket: &mut TcpStream) -> Result<(), String> {
    let mut rng = rand::thread_rng();

    let subscribe = Subscribe {
        packet_id: rng.gen_range(0..9999),
        topic_filters: Vec::from(["temperature".to_string()]),
        requested_qos: Vec::from([0]),
    };

    match subscribe.write_to(socket) {
        Ok(_) => {
            let fixed_header = FixedHeader::read_fixed_header(socket).unwrap();

            match PacketType::from_u8(fixed_header.packet_type) {
                Some(PacketType::Suback) => {
                    let suback = Suback::read_from(socket, fixed_header).unwrap();
                    println!("Received: {:?}", suback);
                    Ok(())
                }
                _ => Err("Unexpected package header (Expected Suback)".to_string()),
            }
        }
        Err(_) => Err("Failed to write package subscribe to socket.".to_string()),
    }
}

fn handle_http_connection(
    mut http_stream: TcpStream,
    measures_manager: Arc<Mutex<MeasuresManager>>,
) {
    let mut buffer = [0; 1024];
    http_stream.read(&mut buffer).unwrap();

    let (status_line, filename) = ("HTTP/1.1 200 OK", "hello.html");

    let contents = fs::read_to_string(filename).unwrap();

    let measures_manager_local = measures_manager.lock().unwrap();
    let temperature_entries = measures_manager_local.get_all_measures();
    drop(measures_manager_local);

    let temperature_table = build_list(&temperature_entries);

    let contents_v2 = contents.replace("{data}", &temperature_table);
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents_v2.len(),
        contents_v2
    );

    http_stream.write(response.as_bytes()).unwrap();
    http_stream.flush().unwrap();
}

fn start_broker_listener(
    mut socket: TcpStream,
    measures_manager: Arc<Mutex<MeasuresManager>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let mut buf = [0u8; 1];
        socket
            .set_read_timeout(Some(Duration::from_millis(TIME_OUT)))
            .unwrap();

        let _ = match socket.peek(&mut buf) {
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                    println!("would have blocked");
                }

                _ => panic!("Got an error: {}", e),
            },
            Ok(_) => {
                match dispatch_packet(&mut socket) {
                    Ok(packet) => {
                        println!("Recibido {:?}", packet);
                        if packet.handle_packet(measures_manager.clone()).is_ok() {};
                    }
                    Err(e) => {
                        println!("Error en dispatch_packet: {}", e)
                    }
                };
            }
        };
        thread::sleep(Duration::from_millis(TIME_CHECK_NEW_MEASURES));
    })
}

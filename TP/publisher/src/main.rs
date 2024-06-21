use rand::Rng;
use shared::packages::connect::Connect;
use shared::packages::packet::WritablePacket;
use shared::packages::publish::Publish;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

static TIME_SEND_NEW_MESSAGE: u64 = 1000;
static SERVER_URL: &str = "localhost:3090";

fn main() {
    if let Ok(mut socket) = TcpStream::connect(SERVER_URL.to_string()) {
        let connect = Connect {
            client_id: "temperature-publisher".to_string(),
            username: "temperature-publisher".to_string(),
            password: "1234".to_string(),
            last_will_topic: "temperature/status".to_string(),
            last_will_message: "temperature publisher stopped working".to_string(),
            keep_alive: 60,
            last_will_qos: 0,
            clean_session: 0,
            last_will_retain: 0,
            last_will_flag: 0_u8,
        };

        if connect.write_to(&mut socket).is_ok() {
            publish_data(&mut socket);
        }
    }
}

fn publish_data(socket: &mut TcpStream) {
    loop {
        let now = SystemTime::now();

        let mut rng = rand::thread_rng();

        let publish = Publish {
            topic_name: "temperature".to_string(),
            payload: rng.gen_range(-30.0..49.1).to_string(),
            packet_id: 0_u16, // QoS 0
            qos: 0,
            retain_flag: 0,
            dup_flag: 0_u8,
        };
        match publish.write_to(socket) {
            Ok(_) => {
                println!("Published {:?} at {:?}", publish, now)
            }
            Err(_) => {
                println!("Failed publishing {:?}", publish)
            }
        }
        thread::sleep(Duration::from_millis(TIME_SEND_NEW_MESSAGE));
    }
}

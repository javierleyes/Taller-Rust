mod config;
mod managers;
mod packages;
mod tests;
mod threadpool;

use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::{MessageManager, PendingMessage};
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::{ClientSubscription, TopicManager};
use crate::packages::packet_dispatcher::dispatch_packet;
use crate::packages::server_packet::PacketError;
use shared::packages::packet::WritablePacket;
use shared::packages::publish::Publish;
use std::cmp;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{event, Level};

static SERVER_ARGS: usize = 2;
static POOL_SIZE: usize = 4;
static TIME_CHECK_NEW_REQUESTS: u64 = 1000;
static TIME_CHECK_NEW_USERS: u64 = 30000;
static TIME_POLL_PENDING_MESSAGES: u64 = 20000;
static CREDENTIALS_FILE: &str = "credentials.txt";

fn main() -> Result<(), String> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        let app_name = &argv[0];
        println!(
            "Invalid amount of arguments (Run with {:?} <config_file>)",
            app_name
        );
        return Err("Aborted due to invalid arguments".to_string());
    }

    let config = config::Config::from_file(&argv[1]).unwrap();
    // uncomment and set an absolute path to the logs directory to store logs. Otherwise use default implementation
    // (writes to stdout)
    //let file_appender = tracing_appender::rolling::hourly("logs", config.log_file);
    let file_appender = std::io::stdout();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::INFO)
        .init();

    let address = "0.0.0.0:".to_owned() + &config.port;

    server_run(&address).unwrap();

    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    event!(Level::INFO, "Server listening on {}", address);

    let session_manager_arc_mutex = Arc::new(Mutex::new(SessionManager::new()));
    let session_manager = Arc::clone(&session_manager_arc_mutex);
    let session_manager_hpm_handle = Arc::clone(&session_manager_arc_mutex);

    let streams_arc_mutex = Arc::new(Mutex::new(Vec::<Socket>::new()));
    let hnr_streams = Arc::clone(&streams_arc_mutex);
    let hnc_streams = Arc::clone(&streams_arc_mutex);
    let hnr_actual_stream = Arc::clone(&streams_arc_mutex);

    let credentials_arc_mutex = Arc::new(Mutex::new(CredentialManager::new()));
    let uc_credential = Arc::clone(&credentials_arc_mutex);
    let hnr_credentials = Arc::clone(&credentials_arc_mutex);

    let topic_manager_arc_mutex = Arc::new(Mutex::new(TopicManager::new()));
    let topic_manager = Arc::clone(&topic_manager_arc_mutex);

    let message_manager_arc_mutex = Arc::new(Mutex::new(MessageManager::new()));
    let message_manager = Arc::clone(&message_manager_arc_mutex);
    let message_manager_hnr_handle = Arc::clone(&message_manager_arc_mutex);

    let new_requests = handle_new_requests(
        hnr_streams,
        hnr_credentials,
        session_manager,
        topic_manager,
        message_manager_hnr_handle,
        hnr_actual_stream,
    );
    let credentials = update_credentials(uc_credential);
    let pending_messages_handle =
        handle_pending_messages(message_manager, session_manager_hpm_handle);

    handle_new_connections(address, hnc_streams).unwrap();

    new_requests.join().unwrap();
    credentials.join().unwrap();
    pending_messages_handle.join().unwrap();

    Ok(())
}

fn update_credentials(credentials: Arc<Mutex<CredentialManager>>) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let mut credential_manager = credentials.lock().unwrap();

        let file = File::open(CREDENTIALS_FILE).unwrap();

        let reader = BufReader::new(file);

        for (_index, line) in reader.lines().enumerate() {
            let line = line.unwrap();

            if line.is_empty() {
                continue;
            }

            let mut pair = line.split(',');
            let username = pair.next().unwrap();
            let password = pair.next().unwrap();

            if !(*credential_manager).has_username(username) {
                (*credential_manager).add_credential(username, password);
            }
        }

        drop(credential_manager);
        thread::sleep(Duration::from_millis(TIME_CHECK_NEW_USERS));
    })
}

fn handle_new_requests(
    active_streams: Arc<Mutex<Vec<Socket>>>,
    credentials: Arc<Mutex<CredentialManager>>,
    sessions: Arc<Mutex<SessionManager>>,
    topics: Arc<Mutex<TopicManager>>,
    messages: Arc<Mutex<MessageManager>>,
    actual_stream: Arc<Mutex<Vec<Socket>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let pool = threadpool::ThreadPool::new(POOL_SIZE).unwrap();

        loop {
            event!(Level::DEBUG, "HNR: Checking for new requests");

            let streams = active_streams.lock().unwrap();

            event!(Level::DEBUG, "HNR: Amount active sockets {}", streams.len());

            for index in 0..streams.len() {
                event!(Level::DEBUG, "HNR: Run socket: {:?}", index);

                let credential_manager = Arc::clone(&credentials);
                let session_manager = Arc::clone(&sessions);
                let topic_session = Arc::clone(&topics);
                let message_manager = Arc::clone(&messages);
                let actual_streams = Arc::clone(&actual_stream);

                let mut buf = [0u8; 100];
                let socket_to_process = streams[index].stream.try_clone().unwrap();

                socket_to_process
                    .set_read_timeout(Some(Duration::from_millis(100)))
                    .unwrap();

                match socket_to_process.peek(&mut buf) {
                    Err(e) => match e.kind() {
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                            event!(
                                Level::DEBUG,
                                "HNR: Socket {:?} would have blocked.",
                                socket_to_process
                            );
                            continue;
                        }
                        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset => {
                            event!(
                                Level::DEBUG,
                                "HNR: Socket {:?} connection is broken. Reason: {:?}",
                                socket_to_process,
                                e
                            );
                            send_last_will(
                                &streams[index].peer,
                                session_manager,
                                topic_session,
                                message_manager,
                            );
                            continue;
                        }
                        _ => panic!("Got an unexpected error: {}", e),
                    },
                    Ok(m) => {
                        if m > 0 {
                            pool.execute(move || {
                                event!(Level::DEBUG, "HNR: New task");

                                let mut mystream = socket_to_process.try_clone().unwrap();
                                match handle_client(
                                    &mut mystream,
                                    credential_manager,
                                    session_manager,
                                    topic_session,
                                    message_manager,
                                    actual_streams,
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        event!(Level::ERROR, "HNR: Error de handler client: {}", e);
                                    }
                                }
                            });
                        }
                    }
                };
            }

            drop(streams);
            thread::sleep(Duration::from_millis(TIME_CHECK_NEW_REQUESTS));
        }
    })
}

fn handle_new_connections(
    address: &str,
    stream_new: Arc<Mutex<Vec<Socket>>>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Ok(address) = stream.peer_addr() {
                    let mut streams = stream_new.lock().unwrap();
                    let socket = Socket {
                        stream: stream.try_clone().unwrap(),
                        peer: address.port(),
                    };
                    streams.push(socket);
                }
            }
            Err(e) => {
                event!(Level::ERROR, "Failed connection: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(
    stream: &mut TcpStream,
    credential_manager: Arc<Mutex<CredentialManager>>,
    session_manager: Arc<Mutex<SessionManager>>,
    topic_manager: Arc<Mutex<TopicManager>>,
    message_manager: Arc<Mutex<MessageManager>>,
    actual_streams: Arc<Mutex<Vec<Socket>>>,
) -> Result<(), PacketError> {
    let packet = dispatch_packet(stream)?;
    event!(Level::INFO, "Server received a package {:?}", packet);

    packet.handle_packet(
        stream,
        credential_manager,
        session_manager,
        topic_manager,
        message_manager,
        actual_streams,
    )
}

fn handle_pending_messages(
    message_manager: Arc<Mutex<MessageManager>>,
    session_manager: Arc<Mutex<SessionManager>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        event!(Level::DEBUG, "MSGMGR: Resending unacknowledged messages");

        let mut message_mgr = message_manager.lock().unwrap();
        let session_mgr = session_manager.lock().unwrap();
        for (client_id, pending_messages) in message_mgr.get_all() {
            for pending_message in pending_messages {
                let mut session = match session_mgr.get_client(client_id) {
                    Some(result) => result,
                    None => {
                        event!(Level::WARN, "Could not get client {:?}", client_id);
                        continue;
                    }
                };
                let publish = pending_message.to_publish_packet();

                match publish.write_to(&mut session.socket.stream) {
                    Ok(_) => event!(
                        Level::INFO,
                        "{:?} was re-sent to client {}",
                        publish,
                        client_id
                    ),
                    Err(e) => event!(
                        Level::WARN,
                        "{:?} could not be sent to client {:?}. Reason: {:?}",
                        publish,
                        client_id,
                        e
                    ),
                }
            }
        }
        drop(message_mgr);
        drop(session_mgr);
        event!(
            Level::DEBUG,
            "MSGMGR: Finished resending unacknowledged messages"
        );
        thread::sleep(Duration::from_millis(TIME_POLL_PENDING_MESSAGES));
    })
}

fn send_last_will(
    peer_addr: &u16,
    session_manager: Arc<Mutex<SessionManager>>,
    topic_manager: Arc<Mutex<TopicManager>>,
    message_manager: Arc<Mutex<MessageManager>>,
) {
    let session_mgr = session_manager.lock().unwrap();
    let mut message_mgr = message_manager.lock().unwrap();
    let mut topic_mgr = topic_manager.lock().unwrap();
    if let Ok(client_id) = session_mgr.get_client_id(peer_addr) {
        event!(
            Level::WARN,
            "Client: {:?} disconnected ungracefully",
            client_id
        );
        if let Some(session) = session_mgr.get_client(&client_id) {
            if let Some(lwt) = session.last_will_testament {
                let lwt_publish = Publish {
                    topic_name: lwt.topic_name,
                    payload: lwt.payload,
                    packet_id: 1_u16, // TODO: generate ids from the server
                    qos: lwt.qos,
                    retain_flag: lwt.retain_flag,
                    dup_flag: 0_u8,
                };
                topic_mgr.update_topic(&lwt_publish);
                let subscriptions: Vec<ClientSubscription> =
                    topic_mgr.get_subscriptions(&lwt_publish.topic_name);
                for sub in subscriptions.iter() {
                    let client_id = sub.client_id.to_string();

                    let mut session = session_mgr.get_client(&client_id).unwrap();

                    event!(Level::DEBUG, "Processing socket {:?}", session.socket);

                    let qos_publish = cmp::min(sub.qos, lwt_publish.qos);

                    if qos_publish != 0 {
                        let pending_message = PendingMessage::from_publish_packet(&lwt_publish);
                        message_mgr.add_message(&client_id, &pending_message);
                    }
                    match lwt_publish.write_to(&mut session.socket.stream) {
                        Ok(_) => event!(
                            Level::INFO,
                            "{:?} sent to client {}",
                            lwt_publish,
                            client_id
                        ),
                        Err(e) => event!(
                            Level::ERROR,
                            "{:?} could not be sent to client {:?}. Reason: {:?}",
                            lwt_publish,
                            client_id,
                            e
                        ),
                    }
                }
            }
        }
    }
    drop(session_mgr);
    drop(message_mgr);
    drop(topic_mgr);
}

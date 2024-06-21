extern crate glib;
extern crate gtk;

mod managers;
mod packages;
mod packagesresponses;
mod tests;
mod windows;

use glib::{Receiver, Sender};
use gtk::prelude::*;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::managers::connectionmanager::ConnectionManager;
use crate::managers::idmanager::IDManager;
use crate::managers::subscriptionmanager::SubscriptionManager;
use crate::packages::packet_dispatcher::dispatch_packet;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use crate::windows::connectwindow::ConnectWindow;
use crate::windows::publishwindow::PublishWindow;
use crate::windows::subscribewindow::SubscribeWindow;

static TIME_OUT: u64 = 100;
static TIME_CHECK_NEW_USERS: u64 = 10000;

fn main() -> Result<(), ()> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let prefix = "suipacha.taller.rust-".to_string();
    let mut application_id = String::new();
    application_id += &prefix;
    application_id += &time;

    let application = gtk::Application::new(Some(&application_id), Default::default());

    application.connect_activate(|app| {
        build_ui(app).unwrap();
    });

    application.run();

    Ok(())
}

fn build_ui(app: &gtk::Application) -> std::io::Result<()> {
    static AVAILABLE_ID_LENGTH: usize = 65535;

    let packet_id_manager_arc_mutex = Arc::new(Mutex::new(IDManager::new(AVAILABLE_ID_LENGTH)));
    let packet_id_manager = Arc::clone(&packet_id_manager_arc_mutex);

    let connection_manager_arc_mutex = Arc::new(Mutex::new(ConnectionManager::new()));
    let connection_manager = Arc::clone(&connection_manager_arc_mutex);

    let subscription_manager_arc_mutex = Arc::new(Mutex::new(SubscriptionManager::new()));
    let subscription_manager = Arc::clone(&subscription_manager_arc_mutex);

    let glade_src = include_str!("ui/src/ui_v2.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window;
    match builder.object("main_window") {
        Some(win) => window = win,
        None => panic!("Error al crear el window principal"),
    };

    window.set_application(Some(app));

    // Channel para el evio de informacion a la interfaz de subscribe
    let (client_sender, client_receiver): (Sender<String>, Receiver<String>) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Channel para el envio del resultado de la conexion del cliente al broker_listener
    let (connection_state_sender, connection_state_receiver): (Sender<u8>, Receiver<u8>) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Channel para el envio del los parametros del connack a la vista
    let (connack_status_sender, connack_status_receiver): (
        Sender<ConnackResponse>,
        Receiver<ConnackResponse>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Channel para el envio del los return_codes del suback a la vista
    let (suback_return_codes_sender, suback_return_codes_receiver): (
        Sender<SubackResponse>,
        Receiver<SubackResponse>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Channel para el envio del resultado del unsuback a la vista
    let (unsuback_status_sender, unsuback_status_receiver): (
        Sender<UnsubackResponse>,
        Receiver<UnsubackResponse>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let connect_window: ConnectWindow;

    match ConnectWindow::new(builder.clone()) {
        Ok(connect_window_out) => connect_window = connect_window_out,
        Err(e) => {
            panic!("Error al crear la ventana connect_window: {}", e)
        }
    }

    connect_window.build(
        connection_manager.clone(),
        subscription_manager.clone(),
        connection_state_sender,
        client_sender.clone(),
        connack_status_receiver,
    );

    let publish_window: PublishWindow;

    match PublishWindow::new(builder.clone()) {
        Ok(publish_window_out) => publish_window = publish_window_out,
        Err(e) => {
            panic!("Error al crear la ventana publish_window: {}", e)
        }
    }

    publish_window.build(connection_manager.clone(), packet_id_manager.clone());

    let subscribe_window: SubscribeWindow;

    match SubscribeWindow::new(builder) {
        Ok(subscribe_window_out) => subscribe_window = subscribe_window_out,
        Err(e) => {
            panic!("Error al crear la ventana subscribe_window: {}", e)
        }
    }

    subscribe_window.build(
        connection_manager.clone(),
        subscription_manager,
        client_receiver,
        suback_return_codes_receiver,
        unsuback_status_receiver,
    );

    connection_state_receiver.attach(None, move |connection_result: u8| {
        if connection_result == 0 {
            start_broker_listener(
                connection_manager.clone(),
                client_sender.clone(),
                connack_status_sender.clone(),
                suback_return_codes_sender.clone(),
                unsuback_status_sender.clone(),
                packet_id_manager.clone(),
            );
        }

        glib::Continue(true)
    });

    window.show_all();
    Ok(())
}

fn start_broker_listener(
    connection_manager: Arc<Mutex<ConnectionManager>>,
    client_sender: glib::Sender<String>,
    connack_status_sender: glib::Sender<ConnackResponse>,
    suback_return_codes_sender: glib::Sender<SubackResponse>,
    unsuback_status_sender: glib::Sender<UnsubackResponse>,
    packet_id_manager: Arc<Mutex<IDManager>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(TIME_CHECK_NEW_USERS));

        let connection_manager_local = connection_manager.lock().unwrap();

        if !connection_manager_local.has_stream() {
            drop(connection_manager_local);
            continue;
        }

        let mut socket: TcpStream;

        match connection_manager_local.get_stream() {
            Ok(stream) => socket = stream,
            Err(_e) => {
                panic!("Error al obtener el stream en start_broker_listener")
            }
        };

        drop(connection_manager_local);
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
                let packet = dispatch_packet(&mut socket).unwrap();
                println!("Recibido {:?}", packet);

                if packet
                    .handle_packet(
                        &mut socket,
                        client_sender.clone(),
                        connack_status_sender.clone(),
                        suback_return_codes_sender.clone(),
                        unsuback_status_sender.clone(),
                        packet_id_manager.clone(),
                    )
                    .is_ok()
                {};
            }
        };
    })
}

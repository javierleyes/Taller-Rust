extern crate gtk;
use crate::managers::connectionmanager::ConnectionManager;
use crate::managers::subscriptionmanager::SubscriptionManager;
use crate::packagesresponses::connackresponse::ConnackResponse;

use glib::clone;
use gtk::prelude::*;
use shared::packages::connect::Connect;
use shared::packages::disconnect::Disconnect;
use shared::packages::packet::WritablePacket;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct ConnectWindow {
    builder: gtk::Builder,
}

impl ConnectWindow {
    pub fn new(builder: gtk::Builder) -> io::Result<Self> {
        Ok(Self { builder })
    }

    pub fn build(
        &self,
        connection_manager: Arc<Mutex<ConnectionManager>>,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
        connection_state_sender: glib::Sender<u8>,
        client_sender: glib::Sender<String>,
        connack_status_receiver: glib::Receiver<ConnackResponse>,
    ) {
        let txt_username: gtk::Entry;
        match self.builder.object("txt_username") {
            Some(txt) => txt_username = txt,
            None => {
                panic!("Error al obtener el username");
            }
        };

        let txt_password: gtk::Entry;
        match self.builder.object("txt_password") {
            Some(txt) => txt_password = txt,
            None => {
                panic!("Error al obtener el password");
            }
        };

        let txt_ip: gtk::Entry;
        match self.builder.object("txt_ip") {
            Some(txt) => txt_ip = txt,
            None => {
                panic!("Error al obtener el ip");
            }
        };

        let txt_puerto: gtk::Entry;
        match self.builder.object("txt_port") {
            Some(txt) => txt_puerto = txt,
            None => {
                panic!("Error al obtener el puerto");
            }
        };

        let txt_last_will_msg: gtk::Entry;
        match self.builder.object("txt_last_will_msg") {
            Some(txt) => txt_last_will_msg = txt,
            None => {
                panic!("Error al obtener el last will msg");
            }
        };

        let txt_last_will_topic: gtk::Entry;
        match self.builder.object("txt_last_will_topic") {
            Some(txt) => txt_last_will_topic = txt,
            None => {
                panic!("Error al obtener el last will topic");
            }
        };

        let rb_clean_session: gtk::RadioButton;
        match self.builder.object("rb_clean_session") {
            Some(rb) => rb_clean_session = rb,
            None => {
                panic!("Error al obtener el rb_clean_session");
            }
        };

        let rb_retain_msg_connect: gtk::RadioButton;
        match self.builder.object("rb_retain_msg_connect") {
            Some(rb) => rb_retain_msg_connect = rb,
            None => {
                panic!("Error al obtener el rb_retain_msg_connect");
            }
        };

        let rb_qos_connect: gtk::RadioButton;
        match self.builder.object("rb_qos_connect") {
            Some(rb) => rb_qos_connect = rb,
            None => {
                panic!("Error al obtener el rb_qos_connect");
            }
        };

        let btn_connect: gtk::Button;
        match self.builder.object("btn_connect") {
            Some(btn) => btn_connect = btn,
            None => {
                panic!("Error al obtener boton connect.");
            }
        };

        let btn_disconnect: gtk::Button;
        match self.builder.object("btn_disconnect") {
            Some(btn) => btn_disconnect = btn,
            None => {
                panic!("Error al obtener boton disconnect.");
            }
        };

        let btn_publish: gtk::Button;
        match self.builder.object("btn_publish") {
            Some(btn) => btn_publish = btn,
            None => {
                panic!("Error al obtener boton publish.");
            }
        };

        let btn_subscribe: gtk::Button;
        match self.builder.object("btn_subscribe") {
            Some(btn) => btn_subscribe = btn,
            None => {
                panic!("Error al obtener boton subscribe.");
            }
        };

        let btn_unsubscribe: gtk::Button;
        match self.builder.object("btn_unsubscribe") {
            Some(btn) => btn_unsubscribe = btn,
            None => {
                panic!("Error al obtener boton unsubscribe.");
            }
        };

        let lbl_connection_state: gtk::Label;
        match self.builder.object("lbl_connection_state") {
            Some(lbl) => lbl_connection_state = lbl,
            None => {
                panic!("Error al obtener label connecction state.");
            }
        };

        btn_disconnect.set_sensitive(false);
        btn_publish.set_sensitive(false);
        btn_subscribe.set_sensitive(false);
        btn_unsubscribe.set_sensitive(false);

        let connection_manager_connect = connection_manager.clone();
        let connection_state_sender_connect = connection_state_sender.clone();
        let builder_connect = self.builder.clone();

        btn_connect.connect_clicked (
            clone!(@weak txt_username, @weak txt_password, @weak txt_ip, @weak txt_puerto,
                    @weak rb_clean_session, @weak lbl_connection_state, @weak txt_last_will_msg,
                    @weak txt_last_will_topic, @weak btn_connect, @weak btn_disconnect,
                    @weak btn_publish, @weak btn_subscribe, @weak btn_unsubscribe,
                    @weak rb_retain_msg_connect, @weak rb_qos_connect => move |_| {

                if txt_username.text().len() == 0 || txt_password.text().len() == 0 {
                    lbl_connection_state.set_text("Error: Complete the required fields.");
                    return;
                }

                if (txt_last_will_topic.text().len() > 0 && txt_last_will_msg.text().len() == 0)
                || (txt_last_will_topic.text().len() == 0 && txt_last_will_msg.text().len() > 0) {
                    lbl_connection_state.set_text("Error: If you want to use the Last Will feature you
                    must complete both Last Will's fields.");
                    return;
                }

                btn_connect.set_sensitive(false);
                lbl_connection_state.set_text(&String::new());

                if txt_ip.text().len() == 0 {
                    txt_ip.set_text("0.0.0.0");
                }

                if txt_puerto.text().len() == 0 {
                    txt_puerto.set_text("3090");
                }

                let mut clean_session_active = 0;
                let mut retain_msg_connect = 0;
                let mut qos_last_will = 1;
                let mut last_will_flag = 0;

                if txt_last_will_topic.text().len() > 0 && txt_last_will_msg.text().len() > 0 {
                    last_will_flag = 1;
                }

                if rb_clean_session.is_active() {
                    clean_session_active = 1;
                }

                if rb_retain_msg_connect.is_active() {
                    retain_msg_connect = 1;
                }

                if rb_qos_connect.is_active() {
                    qos_last_will = 0;
                }

                ConnectWindow::client_connect(builder_connect.clone(), &clean_session_active, &retain_msg_connect, &qos_last_will, &last_will_flag,
                &connection_manager_connect, &connection_state_sender_connect);
            }),
        );

        let connection_manager_disconnect = connection_manager.clone();
        let builder_disconnect = self.builder.clone();

        // Disconnect event
        btn_disconnect.connect_clicked(
            clone!(@weak txt_username, @weak txt_password, @weak txt_ip, @weak txt_puerto,
                    @weak rb_clean_session, @weak lbl_connection_state, @weak txt_last_will_msg,
                    @weak txt_last_will_topic, @weak btn_connect,@weak btn_disconnect, @weak btn_publish,
                    @weak btn_subscribe, @weak btn_unsubscribe  => move |_| {


                    match  ConnectWindow::client_disconnect(&connection_manager_disconnect) {
                        Ok(_) => {
                            txt_password.set_text(&String::new());
                            txt_username.set_text(&String::new());
                            txt_ip.set_text(&String::new());
                            txt_puerto.set_text(&String::new());
                            txt_last_will_msg.set_text(&String::new());
                            txt_last_will_topic.set_text(&String::new());

                            btn_connect.set_sensitive(true);
                            btn_disconnect.set_sensitive(false);
                            btn_publish.set_sensitive(false);
                            btn_subscribe.set_sensitive(false);
                            btn_unsubscribe.set_sensitive(false);

                            lbl_connection_state.set_text("Disconnection successful.");
                            connection_state_sender.send(1).expect("Error al enviar el resultado de la conexion.");
                            client_sender.send((" | ").to_string()).expect("Error al enviar blanqueamiento de topicos al desconectar.");

                            let mut disconnect_manager = connection_manager_disconnect.lock().unwrap();
                            disconnect_manager.drop_stream();
                            drop(disconnect_manager);

                            let mut subscription_manager_local = subscription_manager.lock().unwrap();

                            ConnectWindow::clean_subscriptions_grid(builder_disconnect.clone(), subscription_manager_local.get_active_subscriptions());

                            subscription_manager_local.delete_all_subscription();
                            drop(subscription_manager_local);
                        }
                        Err(e) => {
                            lbl_connection_state.set_text("Disconnection error.");
                            panic!("Error de desconeccion del cliente: {}",e);
                        }
                    }
                }
            ),
        );

        ConnectWindow::update_ui_for_connection_result(
            self.builder.clone(),
            connack_status_receiver,
            connection_manager,
        );
    }

    fn client_connect(
        builder_connect: gtk::Builder,
        clean_session_active: &u8,
        retain_msg_connect: &u8,
        qos_last_will: &u8,
        last_will_flag: &u8,
        connection_manager: &Arc<Mutex<ConnectionManager>>,
        connection_state_sender_connect: &glib::Sender<u8>,
    ) {
        let txt_username: gtk::Entry;
        match builder_connect.object("txt_username") {
            Some(txt) => txt_username = txt,
            None => {
                panic!("Error al obtener el username");
            }
        };

        let txt_password: gtk::Entry;
        match builder_connect.object("txt_password") {
            Some(txt) => txt_password = txt,
            None => {
                panic!("Error al obtener el password");
            }
        };

        let txt_ip: gtk::Entry;
        match builder_connect.object("txt_ip") {
            Some(txt) => txt_ip = txt,
            None => {
                panic!("Error al obtener el ip");
            }
        };

        let txt_puerto: gtk::Entry;
        match builder_connect.object("txt_port") {
            Some(txt) => txt_puerto = txt,
            None => {
                panic!("Error al obtener el puerto");
            }
        };

        let last_will_msg: gtk::Entry;
        match builder_connect.object("txt_last_will_msg") {
            Some(txt) => last_will_msg = txt,
            None => {
                panic!("Error al obtener el last will msg");
            }
        };

        let last_will_topic: gtk::Entry;
        match builder_connect.object("txt_last_will_topic") {
            Some(txt) => last_will_topic = txt,
            None => {
                panic!("Error al obtener el last will topic");
            }
        };

        let spinner: gtk::Spinner;

        match builder_connect.object("gtk_spinner") {
            Some(spin) => spinner = spin,
            None => {
                panic!("Error al obtener el spinner");
            }
        };

        spinner.start();

        let address = txt_ip.text().as_str().to_owned() + ":" + txt_puerto.text().as_str();

        let mut socket: TcpStream;

        match TcpStream::connect(address) {
            Ok(socket_out) => socket = socket_out,
            Err(e) => {
                panic!("Error al crear el socket en client_connect(): {}", e);
            }
        }

        let mut connection = connection_manager.lock().unwrap();

        connection.add_client_stream(socket.try_clone().unwrap());

        drop(connection);

        let connect = Connect {
            client_id: format!("client-{}", txt_username.text().trim()),
            username: txt_username.text().trim().to_owned(),
            password: txt_password.text().trim().to_owned(),
            last_will_topic: last_will_topic.text().to_string(),
            last_will_message: last_will_msg.text().to_string(),
            keep_alive: 60,
            last_will_qos: qos_last_will.to_owned(),
            clean_session: clean_session_active.to_owned(),
            last_will_retain: retain_msg_connect.to_owned(),
            last_will_flag: last_will_flag.to_owned() as u8,
        };

        match connect.write_to(&mut socket) {
            Ok(_) => {
                connection_state_sender_connect
                    .send(0)
                    .expect("Error al enviar el comienzo para escuchar al broker.");
            }
            Err(e) => {
                panic!("Error al enviar el connect: {}", e);
            }
        };
    }

    fn client_disconnect(connection_manager: &Arc<Mutex<ConnectionManager>>) -> Result<u8, String> {
        let mut socket: TcpStream;
        let connection = connection_manager.lock().unwrap();

        match connection.get_stream() {
            Ok(stream) => socket = stream,
            Err(e) => {
                panic!("No hay stream en connectionManager: {}", e)
            }
        };

        drop(connection);

        let disconnect = Disconnect {};

        match disconnect.write_to(&mut socket) {
            Ok(_) => Ok(0),
            Err(e) => Err(e.to_string()),
        }
    }

    fn update_ui_for_connection_result(
        builder: gtk::Builder,
        connack_status_receiver: glib::Receiver<ConnackResponse>,
        connection_manager: Arc<Mutex<ConnectionManager>>,
    ) {
        connack_status_receiver.attach(None, move |connection_result: ConnackResponse| {
            let status_msg: &str;
            let mut error = false;

            let btn_connect: gtk::Button;
            match builder.object("btn_connect") {
                Some(btn) => btn_connect = btn,
                None => {
                    panic!("Error al obtener boton connect.");
                }
            };

            let btn_disconnect: gtk::Button;
            match builder.object("btn_disconnect") {
                Some(btn) => btn_disconnect = btn,
                None => {
                    panic!("Error al obtener boton disconnect.");
                }
            };

            let btn_publish: gtk::Button;
            match builder.object("btn_publish") {
                Some(btn) => btn_publish = btn,
                None => {
                    panic!("Error al obtener boton publish.");
                }
            };

            let btn_subscribe: gtk::Button;
            match builder.object("btn_subscribe") {
                Some(btn) => btn_subscribe = btn,
                None => {
                    panic!("Error al obtener boton subscribe.");
                }
            };

            let btn_unsubscribe: gtk::Button;
            match builder.object("btn_unsubscribe") {
                Some(btn) => btn_unsubscribe = btn,
                None => {
                    panic!("Error al obtener boton unsubscribe.");
                }
            };

            let lbl_connection_state: gtk::Label;
            match builder.object("lbl_connection_state") {
                Some(lbl) => lbl_connection_state = lbl,
                None => {
                    panic!("Error al obtener label connecction state.");
                }
            };

            let spinner: gtk::Spinner;

            match builder.object("gtk_spinner") {
                Some(spin) => spinner = spin,
                None => {
                    panic!("Error al obtener el spinner");
                }
            };

            match connection_result.get_return_code() {
                0 => {
                    match connection_result.get_session_present() {
                        0 => status_msg = "Connection accepted.",
                        _ => status_msg = "Reconnection accepted.",
                    };

                    btn_connect.set_sensitive(false);
                    btn_disconnect.set_sensitive(true);
                    btn_subscribe.set_sensitive(true);
                    btn_unsubscribe.set_sensitive(true);
                    btn_publish.set_sensitive(true);
                }
                1 => {
                    status_msg = "Connection refused, unacceptable protocol version.";
                    error = true;
                }
                2 => {
                    status_msg = "Connection refused, identifier rejected.";
                    error = true;
                }
                3 => {
                    status_msg = "Connection refused, server unavailable.";
                    error = true;
                }
                4 => {
                    status_msg = "Connection refused, bad user name or password.";
                    error = true;
                }
                5 => {
                    status_msg = "Connection refused, not authorized.";
                    error = true;
                }
                _ => {
                    status_msg = "Connection refused, unknown error.";
                    error = true;
                }
            };

            lbl_connection_state.set_text(status_msg);

            if error {
                btn_connect.set_sensitive(true);

                let mut connection = connection_manager.lock().unwrap();
                connection.drop_stream();
                drop(connection);
            }

            spinner.stop();
            glib::Continue(true)
        });
    }

    fn clean_subscriptions_grid(builder: gtk::Builder, active_subscriptions: i32) {
        let grid_subscribe_msg: gtk::Grid;
        match builder.object("grid_subscribe_msg") {
            Some(grid) => grid_subscribe_msg = grid,
            None => {
                panic!("Error al obtener la grilla");
            }
        };

        let txt_topic_to_publish: gtk::Entry;
        match builder.object("txt_topic_to_publish") {
            Some(txt_topic_publish) => {
                txt_topic_to_publish = txt_topic_publish;
                txt_topic_to_publish.set_text(&String::new());
            }
            None => {
                println!("Error al obtener txt_topic_to_publish.");
            }
        };

        let txt_msg_to_publish: gtk::Entry;
        match builder.object("txt_msg_to_publish") {
            Some(txt_msg_publish) => {
                txt_msg_to_publish = txt_msg_publish;
                txt_msg_to_publish.set_text(&String::new());
            }
            None => {
                println!("Error al obtener txt_msg_to_publish.");
            }
        };

        let txt_subscribe_topic: gtk::Entry;
        match builder.object("txt_subscribe_topic") {
            Some(subscribe_topic) => {
                txt_subscribe_topic = subscribe_topic;
                txt_subscribe_topic.set_text(&String::new());
            }
            None => {
                println!("Error al obtener txt_subscribe_topic.");
            }
        };

        for _row_index in 1..active_subscriptions {
            grid_subscribe_msg.remove_row(1);
        }
    }
}

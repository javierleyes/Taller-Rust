extern crate glib;
extern crate gtk;

use crate::managers::connectionmanager::ConnectionManager;
use crate::managers::subscriptionmanager::SubscriptionManager;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;

use glib::clone;
use gtk::prelude::*;
use rand::{thread_rng, Rng};
use shared::packages::packet::WritablePacket;
use shared::packages::subscribe::Subscribe;
use shared::packages::unsubscribe::Unsubscribe;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct SubscribeWindow {
    builder: gtk::Builder,
}

impl SubscribeWindow {
    pub fn new(builder: gtk::Builder) -> io::Result<Self> {
        Ok(Self { builder })
    }

    pub fn build(
        &self,
        connection_manager: Arc<Mutex<ConnectionManager>>,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
        client_receiver: glib::Receiver<String>,
        suback_return_codes_receiver: glib::Receiver<SubackResponse>,
        unsuback_status_receiver: glib::Receiver<UnsubackResponse>,
    ) {
        let lbl_subscribe_state: gtk::Label;
        match self.builder.object("lbl_subscribe_state") {
            Some(lbl) => lbl_subscribe_state = lbl,
            None => {
                panic!("Error al obtener lbl_subscribe_state.")
            }
        };

        let rb_qos_subscribe: gtk::RadioButton;
        match self.builder.object("rb_qos_subscribe") {
            Some(rb) => rb_qos_subscribe = rb,
            None => {
                panic!("Error al obtener rb_qos_subscribe.")
            }
        };

        let txt_subscribe_topic: gtk::Entry;
        match self.builder.object("txt_subscribe_topic") {
            Some(txt) => txt_subscribe_topic = txt,
            None => {
                panic!("Error al obtener txt_subscribe_topic.")
            }
        };

        let btn_subscribe: gtk::Button;
        match self.builder.object("btn_subscribe") {
            Some(btn) => btn_subscribe = btn,
            None => {
                panic!("Error al obtener btn_subscribe.")
            }
        };

        let btn_unsubscribe: gtk::Button;
        match self.builder.object("btn_unsubscribe") {
            Some(btn) => btn_unsubscribe = btn,
            None => {
                panic!("Error al obtener btn_unsubscribe.")
            }
        };

        let spinner_subs: gtk::Spinner;
        match self.builder.object("gtk_spinner_subs") {
            Some(spin) => spinner_subs = spin,
            None => {
                panic!("Error al obtener spinner_subs.")
            }
        };

        let connection_manager_subscribe = connection_manager.clone();

        btn_subscribe.connect_clicked(
            clone!(@weak txt_subscribe_topic, @weak lbl_subscribe_state, @weak rb_qos_subscribe, @weak spinner_subs => move |_| {

                if txt_subscribe_topic.text().len() == 0 {
                    lbl_subscribe_state.set_text("Error: Complete the required fields.");
                    lbl_subscribe_state.set_hexpand(true);
                    return;
                }

                lbl_subscribe_state.set_text("");

                if txt_subscribe_topic.text().contains('+') {
                    if !txt_subscribe_topic.text().contains("/+/") {
                        lbl_subscribe_state.set_text("Error: The wildcard '+' must be between two forward slashes ('/+/')\nand must be in the middle of the topic.");
                        lbl_subscribe_state.set_hexpand(true);
                        return;
                    }
                    else if txt_subscribe_topic.text().ends_with("/+/"){
                        lbl_subscribe_state.set_text("Error: The wildcard '+' can not be at the end of the topic.\nIt must be in the middle.");
                        lbl_subscribe_state.set_hexpand(true);
                        return;
                    }
                }

                let mut qos = 1;

                if rb_qos_subscribe.is_active() {
                    qos = 0;
                }

                SubscribeWindow::client_subscribe(&txt_subscribe_topic.text(), &qos, &connection_manager_subscribe, &spinner_subs);
            }),
        );

        let connection_manager_unsubscribe = connection_manager;
        let subscription_manager_unsubscribe = subscription_manager.clone();
        btn_unsubscribe.connect_clicked(
            clone!(@weak txt_subscribe_topic, @weak lbl_subscribe_state, @weak spinner_subs => move |_| {

                if txt_subscribe_topic.text().len() == 0 {
                    lbl_subscribe_state.set_text("Error: Complete the required fields.");
                    return;
                }

                lbl_subscribe_state.set_text(&String::new());

                SubscribeWindow::client_unsubscribe(&txt_subscribe_topic.text(), &connection_manager_unsubscribe,
                &subscription_manager_unsubscribe, &spinner_subs);
            }),
        );

        SubscribeWindow::update_grid_subscriptions(
            self.builder.clone(),
            client_receiver,
            subscription_manager.clone(),
        );
        SubscribeWindow::update_ui_for_subscribe_result(
            suback_return_codes_receiver,
            lbl_subscribe_state,
            spinner_subs.clone(),
        );
        SubscribeWindow::update_ui_for_unsubscribe_result(
            unsuback_status_receiver,
            self.builder.clone(),
            spinner_subs,
            subscription_manager,
        );
    }

    fn client_subscribe(
        txt_subscribe_topic: &str,
        qos: &u8,
        connection_manager: &Arc<Mutex<ConnectionManager>>,
        spinner: &gtk::Spinner,
    ) {
        spinner.start();
        let mut socket: TcpStream;
        let connection = connection_manager.lock().unwrap();

        match connection.get_stream() {
            Ok(stream) => socket = stream,
            Err(e) => {
                panic!("{}", e)
            }
        };

        drop(connection);

        let mut rng = thread_rng();

        let subscribe = Subscribe {
            packet_id: rng.gen_range(0..9999),
            topic_filters: Vec::from([txt_subscribe_topic.to_owned()]),
            requested_qos: Vec::from([*qos]),
        };

        match subscribe.write_to(&mut socket) {
            Ok(_) => {
                println!("Subscribe enviado: {:?}", subscribe);
            }
            Err(e) => {
                panic!("Error al enviar el subscribe: {}", e)
            }
        };
    }

    fn client_unsubscribe(
        txt_subscribe_topic: &str,
        connection_manager: &Arc<Mutex<ConnectionManager>>,
        subscription_manager: &Arc<Mutex<SubscriptionManager>>,
        spinner: &gtk::Spinner,
    ) {
        spinner.start();
        let mut socket: TcpStream;
        let connection = connection_manager.lock().unwrap();

        match connection.get_stream() {
            Ok(stream) => socket = stream,
            Err(e) => {
                panic!("{}", e)
            }
        };

        let mut rng = thread_rng();

        let unsubscribe = Unsubscribe {
            packet_id: rng.gen_range(1..9999) as u16,
            topic_filters: Vec::from([txt_subscribe_topic.to_owned()]),
        };

        match unsubscribe.write_to(&mut socket) {
            Ok(_) => {
                println!("Unsubscribe enviado: {:?}", unsubscribe);

                let mut subscriptions = subscription_manager.lock().unwrap();
                subscriptions.add_subscription(unsubscribe.packet_id as u16, txt_subscribe_topic);
                drop(subscriptions);
            }
            Err(e) => {
                panic!("Error al enviar el unsubscribe: {}", e)
            }
        };
    }

    fn update_grid_subscriptions(
        builder: gtk::Builder,
        client_receiver: glib::Receiver<String>,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
    ) {
        let topic_col = 0;

        let grid_subscribe_msg: gtk::Grid;
        match builder.object("grid_subscribe_msg") {
            Some(grid) => grid_subscribe_msg = grid,
            None => {
                panic!("Error al obtener la grilla");
            }
        }

        client_receiver.attach(None, move |text: String| {
            let mut publish_info = text.split('|');
            let topic: &str;
            let payload: &str;
            let mut lbl_actual_topic: gtk::Label;
            let mut update_topic = false;

            match publish_info.next() {
                Some(topic_out) => topic = topic_out,
                None => {
                    panic!("Error al obtener el topic para mostrar");
                }
            };

            match publish_info.next() {
                Some(payload_out) => payload = payload_out,
                None => {
                    panic!("Error al obtener el payload para mostrar");
                }
            };

            let mut subscriptions = subscription_manager.lock().unwrap();
            for row_index in 0..subscriptions.get_active_subscriptions() {
                match grid_subscribe_msg.child_at(topic_col, row_index) {
                    Some(topic_label) => match topic_label.downcast() {
                        Ok(grid_topic_label) => {
                            lbl_actual_topic = grid_topic_label;
                            if lbl_actual_topic.text() == topic {
                                SubscribeWindow::update_row(
                                    grid_subscribe_msg.clone(),
                                    payload,
                                    row_index,
                                );
                                update_topic = true;
                                break;
                            }
                        }
                        Err(e) => {
                            println!(
                                "Error al hacer el downcast del topic label de la grilla: {}",
                                e
                            );
                        }
                    },
                    None => {
                        println!("Error al obtener el topic label de la grilla");
                    }
                };
            }

            if !update_topic {
                SubscribeWindow::insert_row(
                    grid_subscribe_msg.clone(),
                    topic,
                    payload,
                    subscriptions.get_active_subscriptions(),
                );
                subscriptions.add_active_subscription();
            };

            grid_subscribe_msg.show_all();
            drop(subscriptions);
            glib::Continue(true)
        });
    }

    fn insert_row(grid_subscribe_msg: gtk::Grid, topic: &str, payload: &str, row: i32) {
        let topic_col = 0;
        let payload_col = 2;

        let lbl_topic = gtk::Label::new(Some(topic));

        grid_subscribe_msg.insert_row(row);
        grid_subscribe_msg.attach(&lbl_topic, topic_col, row, 1, 1);

        // MI PAYLOAD GRID
        let payload_grid = gtk::Grid::new();
        payload_grid.set_row_homogeneous(true);
        payload_grid.set_column_homogeneous(true);
        payload_grid.insert_row(0);
        payload_grid.insert_column(0);

        let mut payload_str = String::new();

        let separator = ("- ").to_string();

        payload_str += &separator;
        payload_str += payload;
        let lbl_payload = gtk::Label::new(Some(&payload_str));
        lbl_payload.set_justify(gtk::Justification::Left);
        payload_grid.attach(&lbl_payload, 0, 0, 1, 1);

        //CREO EL SCROLL WINDOW
        let scrolled_window = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolled_window.set_kinetic_scrolling(true);
        scrolled_window.set_overlay_scrolling(true);
        scrolled_window.set_shadow_type(gtk::ShadowType::EtchedOut);

        //INSERTO MI payload_grid EN EL SCROLLED WINDOW
        scrolled_window.add(&payload_grid);

        //INSERTO MI SCROLL WINDOW EN LA GRILLA GENERAL
        grid_subscribe_msg.attach(&scrolled_window, payload_col, row, 1, 1);
    }

    fn update_row(grid_subscribe_msg: gtk::Grid, payload: &str, row: i32) {
        let payload_col = 2;

        match grid_subscribe_msg.child_at(payload_col, row) {
            Some(scrolled_window) => {
                match scrolled_window.downcast() {
                    Ok(scroll_out) => {
                        let scroll_win: gtk::ScrolledWindow = scroll_out;

                        match scroll_win.children()[0].clone().downcast() {
                            Ok(vp) => {
                                let view_port: gtk::Viewport = vp;

                                match view_port.children()[0].clone().downcast() {
                                    Ok(grid_out) => {
                                        let vp_grid: gtk::Grid = grid_out;

                                        let mut payload_str = String::new();

                                        let separator = ("- ").to_string();
                                        payload_str += &separator;
                                        payload_str += payload;

                                        let lbl_payload = gtk::Label::new(Some(&payload_str));
                                        lbl_payload.set_justify(gtk::Justification::Left);
                                        vp_grid.insert_row(0);
                                        vp_grid.attach(&lbl_payload, 0, 0, 1, 1);
                                    }
                                    Err(_) => {
                                        println!("Error en downcast de view_port");
                                    }
                                };
                            }
                            Err(_) => {
                                println!("Error en downcast de scroll_win");
                            }
                        };
                    }
                    Err(_) => {
                        println!("Error en downcast de scrolled_window");
                    }
                };
            }
            None => {
                println!("Po pude insertar nuevo mensaje");
            }
        };
    }

    fn update_ui_for_subscribe_result(
        suback_return_codes_receiver: glib::Receiver<SubackResponse>,
        lbl_subscribe_state: gtk::Label,
        spinner: gtk::Spinner,
    ) {
        suback_return_codes_receiver.attach(None, move |subscribe_results: SubackResponse| {
            let mut error = false;
            let status_msg: &str;

            for result in &subscribe_results.get_status_codes() {
                if *result == 128_u8 {
                    error = true;
                    break;
                }
            }

            if error {
                status_msg = "Subscription failure.";
            } else {
                status_msg = "Subscription success.";
            };

            lbl_subscribe_state.set_text(status_msg);
            spinner.stop();
            glib::Continue(true)
        });
    }

    fn update_ui_for_unsubscribe_result(
        unsuback_status_receiver: glib::Receiver<UnsubackResponse>,
        builder: gtk::Builder,
        spinner: gtk::Spinner,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
    ) {
        let lbl_subscribe_state: gtk::Label = builder.object("lbl_subscribe_state").unwrap();

        unsuback_status_receiver.attach(None, move |unsubscribe_result: UnsubackResponse| {
            let status_msg: &str;

            if unsubscribe_result.get_status_code() == 1 {
                status_msg = "Unsubscription failure.";
            } else {
                status_msg = "Unsubscription success.";
                let topic_col = 0;
                let mut lbl_actual_topic: gtk::Label;
                let grid_subscribe_msg: gtk::Grid;

                match builder.object("grid_subscribe_msg") {
                    Some(grid) => grid_subscribe_msg = grid,
                    None => {
                        panic!("Error al obtener la grilla");
                    }
                };

                let mut subscriptions = subscription_manager.lock().unwrap();

                for row_index in 0..subscriptions.get_active_subscriptions(){

                    match grid_subscribe_msg.child_at(topic_col, row_index){
                        Some(topic_label) => match topic_label.downcast() {
                            Ok(grid_topic_label) => {
                                lbl_actual_topic = grid_topic_label;

                                match subscriptions.get_packet_topic(unsubscribe_result.get_packet_id()) {
                                    Ok(topic_out) => {

                                        if lbl_actual_topic.text() ==  topic_out{

                                            grid_subscribe_msg.remove_row(row_index);
                                            subscriptions.remove_active_subscription();
                                            subscriptions.delete_subscription(unsubscribe_result.get_packet_id());
                                            drop(subscriptions);
                                            grid_subscribe_msg.show_all();
                                            break;
                                        };
                                    },
                                    Err(e) => {
                                        println!("Error al obtener el topic del manager: {}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                println!("Error al hacer el downcast del topic label de la grilla al desuscribirse: {}", e); 
                            }
                        },
                        None => {
                            println!("Error al obtener el topic label de la grilla al desuscribirse"); 
                        }
                    };
                }
            };

            lbl_subscribe_state.set_text(status_msg);
            spinner.stop();
            glib::Continue(true)
        });
    }
}

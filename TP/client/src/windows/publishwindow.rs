extern crate gtk;
use crate::managers::connectionmanager::ConnectionManager;
use crate::managers::idmanager::IDManager;

use glib::clone;
use gtk::prelude::*;
use shared::packages::packet::WritablePacket;
use shared::packages::publish::Publish;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct PublishWindow {
    builder: gtk::Builder,
}

impl PublishWindow {
    pub fn new(builder: gtk::Builder) -> io::Result<Self> {
        Ok(Self { builder })
    }

    pub fn build(
        &self,
        connection_manager: Arc<Mutex<ConnectionManager>>,
        packet_id_manager: Arc<Mutex<IDManager>>,
    ) {
        let txt_topic_to_publish: gtk::Entry;
        match self.builder.object("txt_topic_to_publish") {
            Some(txt) => txt_topic_to_publish = txt,
            None => {
                panic!("Error al obtener el txt_topic_to_publish.");
            }
        };

        let txt_msg_to_publish: gtk::Entry;
        match self.builder.object("txt_msg_to_publish") {
            Some(txt) => txt_msg_to_publish = txt,
            None => {
                panic!("Error al obtener el txt_msg_to_publish.");
            }
        };

        let lbl_publish_state: gtk::Label;
        match self.builder.object("lbl_publish_state") {
            Some(lbl) => lbl_publish_state = lbl,
            None => {
                panic!("Error al obtener el lbl_publish_state.");
            }
        };

        let rb_retain_msg: gtk::RadioButton;
        match self.builder.object("rb_retain_msg") {
            Some(rb) => rb_retain_msg = rb,
            None => {
                panic!("Error al obtener el rb_retain_msg.");
            }
        };

        let rb_qos_publish: gtk::RadioButton;
        match self.builder.object("rb_qos_publish") {
            Some(rb) => rb_qos_publish = rb,
            None => {
                panic!("Error al obtener el rb_qos_publish.");
            }
        };

        let btn_publish: gtk::Button;
        match self.builder.object("btn_publish") {
            Some(btn) => btn_publish = btn,
            None => {
                panic!("Error al obtener el btn_publish.");
            }
        };

        btn_publish.connect_clicked(
            clone!(@weak txt_topic_to_publish, @weak txt_msg_to_publish,
                    @weak rb_retain_msg, @weak lbl_publish_state => move |_| {

                if txt_topic_to_publish.text().len() == 0 || txt_msg_to_publish.text().len() == 0 {
                    lbl_publish_state.set_text("Error: Complete the required fields.");
                    return;
                }

                let mut retain_mge_active = 0;
                let mut qos = 1;

                if rb_retain_msg.is_active() {
                    retain_mge_active = 1;
                }

                if rb_qos_publish.is_active() {
                    qos = 0;
                }

                match PublishWindow::client_publish(&txt_topic_to_publish.text(), &txt_msg_to_publish.text(),
                &retain_mge_active, &qos, &connection_manager, &packet_id_manager) {
                    Ok(result) => {
                        if result == 0 {
                            lbl_publish_state.set_text("Publication successful.");
                        }
                        else{
                            lbl_publish_state.set_text("Publication error.")
                        };
                    },
                    Err(e) => {
                        println!("Error en el resultado de client_publish: {}", e);
                    }
                };


            }),
        );
    }

    fn client_publish(
        txt_topic_to_publish: &str,
        txt_msg_to_publish: &str,
        retain_mge_active: &u8,
        qos_publish: &u8,
        connection_manager: &Arc<Mutex<ConnectionManager>>,
        packet_id_manager: &Arc<Mutex<IDManager>>,
    ) -> Result<u32, u32> {
        let mut socket: TcpStream;
        let connection = connection_manager.lock().unwrap();
        let mut idmanager = packet_id_manager.lock().unwrap();

        match connection.get_stream() {
            Ok(stream) => socket = stream,
            Err(e) => {
                panic!("{}", e)
            }
        };

        drop(connection);

        let packet_id_local: u16;
        if *qos_publish == 0 {
            packet_id_local = 0_u16;
        } else {
            packet_id_local = idmanager.take_id();
        }
        drop(idmanager);
        let publish = Publish {
            topic_name: txt_topic_to_publish.to_owned(),
            payload: txt_msg_to_publish.to_owned(),
            retain_flag: retain_mge_active.to_owned(),

            packet_id: packet_id_local,
            qos: qos_publish.to_owned(),
            dup_flag: 0,
        };

        match publish.write_to(&mut socket) {
            Ok(_) => println!("Publish enviado: {:?}", publish),
            Err(e) => {
                panic!("Error al enviar el publish: {}", e)
            }
        };
        Ok(0)
    }
}

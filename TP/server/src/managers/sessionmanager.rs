use std::collections::HashMap;
use std::net::TcpStream;
use tracing::{event, Level};

#[derive(Debug, Clone)]
pub struct LastWillTestament {
    /// A String containing the topic to publish to
    pub topic_name: String,
    /// A payload String that represents the message to be published
    pub payload: String,
    /// The QoS level for this packet
    pub qos: u8,
    /// A retain_flag that indicates if the message should be retained or not
    pub retain_flag: u8,
}

pub struct Session {
    pub client_id: String,
    pub socket: Socket,
    pub last_will_testament: Option<LastWillTestament>,
}

#[derive(Debug)]
pub struct Socket {
    pub stream: TcpStream,
    pub peer: u16,
}

/// This struct represents a storage of client id and their properties
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    peer_client: HashMap<u16, String>,
}

impl SessionManager {
    /// Returns an empty SessionManager
    pub fn new() -> SessionManager {
        SessionManager {
            sessions: HashMap::new(),
            peer_client: HashMap::new(),
        }
    }

    /// Add a client to sessions
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client_id to add
    /// * `stream` - A stream
    ///
    pub fn add_client(
        &mut self,
        client_id: &str,
        stream: TcpStream,
        lwt: Option<LastWillTestament>,
    ) {
        match stream.try_clone() {
            Ok(stream) => match stream.peer_addr() {
                Ok(peer_address) => {
                    let socket = Socket {
                        stream,
                        peer: peer_address.port(),
                    };

                    self.sessions.insert(
                        client_id.to_string(),
                        Session {
                            client_id: client_id.to_string(),
                            socket,
                            last_will_testament: lwt,
                        },
                    );
                    self.peer_client
                        .insert(peer_address.port(), client_id.to_string());
                }
                Err(e) => {
                    event!(Level::ERROR, "Client could not be added. Reason: {:?}", e)
                }
            },
            Err(e) => {
                event!(Level::ERROR, "Client could not be added. Reason: {:?}", e)
            }
        }
    }

    /// Checks if a given client id exists in the SessionManager
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client id to search
    ///
    pub fn has_client(&self, client_id: &str) -> bool {
        self.sessions.contains_key(client_id)
    }

    /// Checks if a given peer exists in the SessionManager
    /// # Arguments
    ///
    /// * `peer` - A string slice containing the cpeer to search
    ///
    pub fn has_peer(&self, peer: &u16) -> bool {
        self.peer_client.contains_key(peer)
    }

    /// Returns the stream
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client_id to get the stream for
    ///
    pub fn get_client(&self, client_id: &str) -> Option<Session> {
        if self.has_client(client_id) {
            match self.sessions.get(client_id) {
                Some(session) => match session.socket.stream.try_clone() {
                    Ok(cloned_stream) => Some(Session {
                        client_id: session.client_id.to_string(),
                        socket: Socket {
                            stream: cloned_stream,
                            peer: session.socket.peer,
                        },
                        last_will_testament: session.last_will_testament.clone(),
                    }),
                    Err(e) => {
                        event!(
                            Level::ERROR,
                            "Get client failed cloning stream. Reason {:?}",
                            e
                        );
                        None
                    }
                },
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns the clientid
    /// # Arguments
    ///
    /// * `peer` - A string slice containing the peer to get the clientid for
    ///
    pub fn get_client_id(&self, peer: &u16) -> Result<String, String> {
        if self.has_peer(peer) {
            match self.peer_client.get(peer) {
                Some(peer) => Ok(peer.to_string()),
                _ => Err("This client does not have an active peer".to_string()),
            }
        } else {
            Err("Failed getting client id...".to_string())
        }
    }

    /// Delete a client from the sessions
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client id
    ///
    pub fn delete(&mut self, client_id: &str) {
        if self.has_client(client_id) {
            match self.sessions.get(client_id) {
                Some(session) => match session.socket.stream.try_clone() {
                    Ok(stream) => match stream.peer_addr() {
                        Ok(peer) => {
                            self.sessions.remove(client_id);
                            self.peer_client.remove(&peer.port());
                        }
                        Err(e) => {
                            event!(Level::ERROR, "There was a problem in delete {:?}", e);
                        }
                    },
                    Err(e) => {
                        event!(Level::ERROR, "There was a problem in delete {:?}", e);
                    }
                },
                None => {
                    event!(Level::ERROR, "There was a problem in delete");
                }
            }
        }
    }

    /// Replace a tcpstream for client
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client id
    /// * `stream` - A TcpStream
    ///
    pub fn replace_stream(&mut self, client_id: &str, new_stream: TcpStream) {
        if self.has_client(client_id) {
            match self.sessions.get_mut(client_id) {
                Some(session) => {
                    self.peer_client.remove(&session.socket.peer);

                    match new_stream.peer_addr() {
                        Ok(peer_address) => {
                            self.peer_client
                                .insert(peer_address.port(), client_id.to_string());
                        }
                        Err(e) => {
                            event!(Level::ERROR, "Client could not be replace. Reason: {:?}", e);
                        }
                    }
                }
                _ => event!(
                    Level::ERROR,
                    "Client {:?} does not have an active stream",
                    client_id
                ),
            }

            if let Some(removed_session) = self.sessions.remove(client_id) {
                let last_will_testament = removed_session.last_will_testament;
                self.add_client(client_id, new_stream, last_will_testament);
            }
        } else {
            event!(Level::ERROR, "The client {:?} does not exist", client_id);
        }
    }

    /// Get the old peer associate to client
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client id
    ///
    pub fn get_old_peer(&mut self, client_id: &str) -> Result<u16, String> {
        if self.has_client(client_id) {
            match self.sessions.get(client_id) {
                Some(session) => Ok(session.socket.peer),
                None => Err("Get old peer failed.".to_string()),
            }
        } else {
            Err("The client id doesn't exist".to_string())
        }
    }
}

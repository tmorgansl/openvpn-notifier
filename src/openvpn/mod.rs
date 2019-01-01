use crate::conf;
use crate::dispatcher;
use openvpn_management::{Client, EventManager};
use std::collections::HashMap;

const CRITICAL_FAILURE_COUNT: usize = 3;

pub trait ClientController {
    fn update_connected_clients(&mut self);
}

struct TCPController<'a> {
    dispatcher: &'a dispatcher::Dispatcher,
    manager: Box<openvpn_management::EventManager>,
    clients: HashMap<String, Client>,
    failed_calls: usize,
}

impl<'a> ClientController for TCPController<'a> {
    fn update_connected_clients(&mut self) {
        let status = match self.manager.get_status() {
            Ok(c) => {
                self.failed_calls = 0;
                c
            }
            Err(e) => {
                self.failed_calls += 1;
                eprintln!(
                    "failed call to openvpn server {}, failed count: {}",
                    e, self.failed_calls
                );
                if self.failed_calls == CRITICAL_FAILURE_COUNT {
                    self.dispatcher.alert(format!("{} consecutive failed calls to openvpn server, please check the error logs", CRITICAL_FAILURE_COUNT));
                }
                return;
            }
        };

        let new_clients = clients_to_hashmap(status.clients());

        for (name, client) in &self.clients {
            match new_clients.get(name) {
                Some(_) => {}
                None => self.dispatcher.client_disconnected(client),
            }
        }

        for (name, client) in &new_clients {
            match self.clients.get(name) {
                Some(_) => {}
                None => self.dispatcher.client_connected(client),
            }
        }

        self.clients = new_clients;
    }
}

pub fn new<'a>(
    config: &conf::Config,
    dispatcher: &'a dispatcher::Dispatcher,
) -> impl ClientController + 'a {
    let mut connection_string = config.openvpn.address.clone();
    connection_string.push_str(":");
    connection_string.push_str(&mut config.openvpn.port.to_string());

    let mut event_manager = openvpn_management::CommandManagerBuilder::new()
        .management_url(&connection_string)
        .build();
    let status = event_manager
        .get_status()
        .expect("could not get initial clients from openvpn server");
    TCPController {
        dispatcher: dispatcher,
        manager: Box::new(event_manager),
        clients: clients_to_hashmap(status.clients()),
        failed_calls: 0,
    }
}

fn clients_to_hashmap(clients: &Vec<openvpn_management::Client>) -> HashMap<String, Client> {
    let mut clients_map = HashMap::new();
    for client in clients {
        clients_map.insert(client.name().to_owned(), (*client).clone());
    }
    clients_map
}

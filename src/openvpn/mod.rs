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
    connection_string.push_str(&config.openvpn.port.to_string());

    let mut event_manager = openvpn_management::CommandManagerBuilder::new()
        .management_url(&connection_string)
        .build();
    let status = event_manager
        .get_status()
        .expect("could not get initial clients from openvpn server");
    TCPController {
        dispatcher,
        manager: Box::new(event_manager),
        clients: clients_to_hashmap(status.clients()),
        failed_calls: 0,
    }
}

fn clients_to_hashmap(clients: &[openvpn_management::Client]) -> HashMap<String, Client> {
    let mut clients_map = HashMap::new();
    for client in clients {
        clients_map.insert(client.name().to_owned(), (*client).clone());
    }
    clients_map
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::{DateTime, TimeZone, Utc};
    use dispatcher::Dispatcher;
    use openvpn_management::{Client, OpenvpnError, Result, Status};
    use simulacrum::*;

    create_mock! {
        impl EventManager for EventManagerMock (self) {
            expect_get_status("get_status"):
            fn get_status(&mut self) -> Result<Status>;
        }
    }

    create_mock! {
        impl Dispatcher for DispatcherMock (self) {
            expect_client_connected("client_connected"):
            fn client_connected(&self, client: &Client);

            expect_client_disconnected("client_disconnected"):
            fn client_disconnected(&self, client: &Client);

            expect_alert("alert"):
            fn alert(&self, body: String);
        }
    }

    fn new_mock_client(
        name: &'static str,
        ip_address: &'static str,
        epoch_seconds: i64,
        bytes_received: f64,
        bytes_sent: f64,
    ) -> Client {
        let datetime: DateTime<Utc> = Utc.timestamp(epoch_seconds, 0);
        Client::new(
            name.to_string(),
            ip_address.to_string(),
            datetime,
            bytes_received,
            bytes_sent,
        )
    }

    #[test]
    fn test_client_connected() {
        let mut event_manager = EventManagerMock::new();
        let mut dispatcher = DispatcherMock::new();

        let mut expected_clients = Vec::new();
        let client = new_mock_client("test-client", "127.0.0.1", 1_546_277_714, 100.0, 200.0);

        expected_clients.push(client);

        let status = Status::new(expected_clients);

        event_manager
            .expect_get_status()
            .called_once()
            .returning(move |_| Ok(status.clone()));

        dispatcher.expect_client_connected().called_once();

        let mut tcp_controller = TCPController {
            dispatcher: &dispatcher,
            manager: Box::new(event_manager),
            clients: HashMap::new(),
            failed_calls: 0,
        };

        tcp_controller.update_connected_clients();
    }

    #[test]
    fn test_client_disconnected() {
        let mut event_manager = EventManagerMock::new();
        let mut dispatcher = DispatcherMock::new();

        let mut current_clients = HashMap::new();

        let client = new_mock_client("test-client", "127.0.0.1", 1_546_277_714, 100.0, 200.0);
        current_clients.insert("test-client".to_string(), client);

        let status = Status::new(Vec::new());

        event_manager
            .expect_get_status()
            .called_once()
            .returning(move |_| Ok(status.clone()));

        dispatcher.expect_client_disconnected().called_once();

        let mut tcp_controller = TCPController {
            dispatcher: &dispatcher,
            manager: Box::new(event_manager),
            clients: current_clients,
            failed_calls: 0,
        };

        tcp_controller.update_connected_clients();
    }

    #[test]
    fn test_client_alert() {
        let mut event_manager = EventManagerMock::new();
        let mut dispatcher = DispatcherMock::new();

        event_manager
            .expect_get_status()
            .called_once()
            .returning(move |_| {
                Err(OpenvpnError::MalformedResponse(
                    "something bad happened".to_string(),
                ))
            });

        dispatcher.expect_alert().called_once();

        let mut tcp_controller = TCPController {
            dispatcher: &dispatcher,
            manager: Box::new(event_manager),
            clients: HashMap::new(),
            failed_calls: CRITICAL_FAILURE_COUNT - 1,
        };

        tcp_controller.update_connected_clients();
    }
}

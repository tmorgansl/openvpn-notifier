use crate::conf;
use crate::dispatcher;
use chrono::prelude::{DateTime, Local, TimeZone};
use std::boxed::Box;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, Write};
use std::net::TcpStream;
use time::Duration;

const ENDING: &str = "END";
const START_LINE: &str = "CLIENT_LIST";
const UNDEF: &str = "UNDEF";
const CRITICAL_FAILURE_COUNT: usize = 3;

pub struct Client {
    pub name: String,
    pub address: String,
    pub connected_since: DateTime<Local>,
    pub duration: Duration,
    pub bytes_received: f64,
    pub bytes_sent: f64,
}

pub trait ClientController {
    fn update_connected_clients(&mut self);
}

struct TCPController {
    connection_string: String,
    dispatcher: Box<dispatcher::Dispatcher>,
    clients: HashMap<String, Client>,
    failed_calls: usize,
}

impl ClientController for TCPController {
    fn update_connected_clients(&mut self) {
        let new_clients = match get_new_clients(&self.connection_string) {
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

pub fn new(
    config: &conf::Config,
    dispatcher: impl dispatcher::Dispatcher + 'static,
) -> impl ClientController {
    let mut connection_string = config.openvpn.address.clone();
    connection_string.push_str(":");
    connection_string.push_str(&mut config.openvpn.port.to_string());
    let clients = get_new_clients(&connection_string).expect(&format!(
        "could not get initial clients from openvpn server at address {}",
        connection_string
    ));
    TCPController {
        connection_string: connection_string,
        dispatcher: Box::new(dispatcher),
        clients: clients,
        failed_calls: 0,
    }
}

fn get_new_clients(connection_string: &str) -> Result<HashMap<String, Client>, Error> {
    let mut stream = TcpStream::connect(connection_string)?;
    stream.write("status\n".as_bytes())?;
    let mut reader = BufReader::new(&stream);

    let mut line = String::new();
    while !line.trim().ends_with(ENDING) {
        reader.read_line(&mut line).expect("Could not read");
    }

    Ok(parse_status_output(line))
}

fn parse_status_output(output: String) -> HashMap<String, Client> {
    let split = output.split("\n");
    let mut map = HashMap::new();
    for s in split {
        let line = String::from(s);
        if line.starts_with(START_LINE) {
            let client = parse_client(line);
            if client.name != UNDEF {
                map.insert(client.name.clone(), client);
            }
        }
    }
    map
}

fn parse_client(raw_client: String) -> Client {
    let split = raw_client.split("\t");
    let vec = split.collect::<Vec<&str>>();
    let name = vec[1];
    let address = vec[2].split(":").next().expect("malformed ip address");
    let timestamp = vec[8].parse::<i64>().unwrap();
    let bytes_received = vec[5].parse::<f64>().unwrap();
    let bytes_sent = vec[6].parse::<f64>().unwrap();
    Client {
        name: String::from(name),
        address: String::from(address),
        connected_since: get_local_start_time(timestamp),
        duration: get_duration(timestamp),
        bytes_received: bytes_received,
        bytes_sent: bytes_sent,
    }
}

fn get_local_start_time(timestamp: i64) -> DateTime<Local> {
    let datetime: DateTime<Local> = Local.timestamp(timestamp, 0);
    datetime
}

fn get_duration(timestamp: i64) -> Duration {
    let connected_at = get_local_start_time(timestamp);
    let now: DateTime<Local> = Local::now();
    now.signed_duration_since(connected_at)
}

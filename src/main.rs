extern crate pushover;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate pretty_bytes;

mod conf;
mod dispatcher;
mod openvpn;

use crate::openvpn::ClientController;
use std::{thread, time};

fn main() {
    let config = conf::get_config();
    let dispatcher = dispatcher::new(&config);
    let mut controller = openvpn::new(&config, &dispatcher);
    loop {
        controller.update_connected_clients();
        thread::sleep(time::Duration::from_millis(5000));
    }
}

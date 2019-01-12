extern crate chrono;
extern crate pushover;
#[macro_use]
extern crate clap;
extern crate openvpn_management;
extern crate pretty_bytes;
#[cfg(test)]
extern crate simulacrum;

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

use clap::{App, Arg};

const TOKEN: &str = "token";
const USER_KEY: &str = "user_key";
const ADDRESS: &str = "address";
const PORT: &str = "port";

pub struct Config {
    pub openvpn: Openvpn,
    pub pushover: Pushover,
}

pub struct Openvpn {
    pub address: String,
    pub port: u16,
}

pub struct Pushover {
    pub token: String,
    pub user_key: String,
}

pub fn get_config() -> Config {
    let matches = App::new("Openvpn Notifier")
        .version(crate_version!())
        .about("Sends notifications to pushover with client updates from an openvpn server")
        .arg(
            Arg::with_name(TOKEN)
                .short("t")
                .long("token")
                .value_name("TOKEN")
                .help("Pushover application token")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(USER_KEY)
                .short("uk")
                .long("user_key")
                .value_name("USER KEY")
                .help("Pushover user key")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ADDRESS)
                .short("s")
                .long("server")
                .value_name("IP")
                .help("Address of openvpn server")
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(PORT)
                .short("p")
                .long("port")
                .value_name("port")
                .help("Management port for openvpn server")
                .default_value("5555")
                .takes_value(true),
        )
        .get_matches();

    Config {
        openvpn: Openvpn {
            address: String::from(matches.value_of(ADDRESS).unwrap()),
            port: matches.value_of(PORT).unwrap().parse::<u16>().unwrap(),
        },
        pushover: Pushover {
            token: String::from(matches.value_of(TOKEN).unwrap()),
            user_key: String::from(matches.value_of(USER_KEY).unwrap()),
        },
    }
}

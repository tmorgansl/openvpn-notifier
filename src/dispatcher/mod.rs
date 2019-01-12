use crate::conf;
use chrono::prelude::{DateTime, Local, Utc};
use chrono::Duration;
use openvpn_management::Client;
use pretty_bytes::converter::convert;
use pushover::requests::message::SendMessage;
use pushover::{SyncAPI, SyncAPIBuilder};

pub trait Dispatcher {
    fn client_connected(&self, client: &Client);
    fn client_disconnected(&self, client: &Client);
    fn alert(&self, body: String);
}

struct Pushover {
    api: SyncAPI,
    token: String,
    user_key: String,
}

impl Dispatcher for Pushover {
    fn client_connected(&self, client: &Client) {
        let date_string = client
            .connected_since()
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S");
        let body = format!(
            "client {} has connected from ip address {} on {} local time",
            client.name(),
            client.ip_address(),
            date_string
        );
        self.alert(body);
    }

    fn client_disconnected(&self, client: &Client) {
        let duration = get_duration(client.connected_since());
        let body = format!("client {} has disconnected. They received {} of data and sent {} of data. Their session lasted approximately {}",
        client.name(),
        convert(client.bytes_received()),
        convert(client.bytes_sent()),
        parse_duration(&duration));
        self.alert(body);
    }

    fn alert(&self, body: String) {
        let msg = SendMessage::new(self.token.clone(), self.user_key.clone(), body.clone());
        match self.api.send(&msg) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("error sending message: {} to pushover {}", body, e);
            }
        };
    }
}

pub fn new(config: &conf::Config) -> impl Dispatcher {
    let api = SyncAPIBuilder::new().build().expect("Error creating API");
    Pushover {
        api,
        token: config.pushover.token.clone(),
        user_key: config.pushover.user_key.clone(),
    }
}

fn get_duration(start_time: &DateTime<Utc>) -> Duration {
    let now: DateTime<Utc> = Utc::now();
    now.signed_duration_since(start_time.clone())
}

fn parse_duration(duration: &Duration) -> String {
    let num_seconds = duration.num_seconds();
    let mut units = "seconds";
    let mut formated_value = num_seconds as f64;
    if num_seconds >= 3600 {
        formated_value = num_seconds as f64 / 3600.0;
        units = "hours";
    } else if num_seconds >= 60 {
        formated_value = num_seconds as f64 / 60.0;
        units = "minutes";
    }
    format!("{:.1} {}", formated_value, units)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_parse_duration_seconds() {
        let connect_time = Utc.ymd(2019, 1, 1).and_hms(1, 0, 30);
        let now = Utc.ymd(2019, 1, 1).and_hms(1, 0, 40);
        let duration = now.signed_duration_since(connect_time);

        let output = parse_duration(&duration);

        assert_eq!("10.0 seconds", output);
    }

    #[test]
    fn test_parse_duration_minutes() {
        let connect_time = Utc.ymd(2019, 1, 1).and_hms(1, 0, 30);
        let now = Utc.ymd(2019, 1, 1).and_hms(1, 10, 30);
        let duration = now.signed_duration_since(connect_time);

        let output = parse_duration(&duration);

        assert_eq!("10.0 minutes", output);
    }

    #[test]
    fn test_parse_duration_hours() {
        let connect_time = Utc.ymd(2019, 1, 1).and_hms(1, 0, 30);
        let now = Utc.ymd(2019, 1, 1).and_hms(11, 0, 30);
        let duration = now.signed_duration_since(connect_time);

        let output = parse_duration(&duration);

        assert_eq!("10.0 hours", output);
    }
}

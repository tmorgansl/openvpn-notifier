# Openvpn Notifier

Sends notifications to [pushover](https://pushover.net/) when a client connects or disconnects from an openvpn server. The application is written in rust, purely as a learning exercise to get used to writing rust code.

### Disclaimer

It is strongly encouraged to only run this against openvpn servers which have only a few clients. Only one request to the pushover API will ever be made at one time, however if a lot of clients connect or disconnect at the same time it could result in a lot of messages being sent one after another. My use case was a raspberry pi openvpn server on my home network with a handful of regular clients. Always [be friendly](https://pushover.net/api#friendly) to the pushover API.

### Prerequisites

You will need:-
* A running openvpn server with access to its management port
* A [pushover account](https://pushover.net/login)
* A user key and application token from pushover

To compile from source you will require [Rust stable](https://www.rust-lang.org/tools/install) and cargo

### Installing

The easiest way to install the application is to pull the latest binary from the [release page](https://github.com/tmorgansl/openvpn-notifier/releases)

#### Complile from Source
Cargo can be used to install the application from source

```
git clone git@github.com:tmorgansl/openvpn-notifier.git
cd openvpn_notifier
cargo build --release
```

This will create the target binary in `target/release`.

The installation has been tested on stable linux toolchain, v1.31.1.

### Running

The binary file requires your application token and user key passed as command line arguments

```
./openvpn-notifier --token=<TOKEN> --user_key=<USER_KEY>
```

Once the application is running, it will poll the openvpn management address (default `localhost:5555`) every 5 seconds for changes in the number of clients.

Additional options can be found by using the help flag

```
./openvpn-notifier -h
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details


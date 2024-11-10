mod tools;

use tools::*;

const DEFAULT_PORT: u16 = 5000;
const DEFAULT_TIMEOUT: u64 = 500;

fn main() {

    let json = JSON::new().unwrap_or_stderr();

    let connection = {
        
        let ip = json.get_or::<&str>(&["network_args", "ip"], "");
        let port = json.get_or::<u16>(&["network_args", "port"], DEFAULT_PORT);
        let timeout = json.get_or::<u64>(&["network_args", "timeout"], DEFAULT_TIMEOUT);

        Connection::new(ip, port, timeout).unwrap_or_stderr()

    };

    format!("Connection established with: {}", connection.peer_addr().unwrap()).send_to_stdout();

}
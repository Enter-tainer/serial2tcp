use anyhow::{Ok, Result};
use clap::Parser;
use tokio::net::TcpSocket;
use tokio_serial::SerialPortBuilderExt;
#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}
#[derive(clap::Subcommand, Debug)]
enum Action {
    #[clap(name = "ls", about = "list all available serial ports")]
    List,
    #[clap(name = "connect", about = "connect given port to server")]
    Connect {
        #[clap(required = true, help = "ip address and port, e.g. 1.1.1.1:1080")]
        addr: String,
        #[clap(required = true, help = "name of serial port, e.g. /dev/ttyUSB1 for Linux, COM1 for Windows")]
        serial_port: String,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if matches!(args.action, Action::List) {
        let t = tokio_serial::available_ports()?;
        for i in t {
            println!("{}", i.port_name);
        }
        return Ok(());
    }
    if let Action::Connect {
        addr,
        serial_port: port,
    } = args.action
    {
        let tcp_socket = TcpSocket::new_v4()?;
        let mut tcp_stream = tcp_socket.connect(addr.parse()?).await?;
        let mut serial_steam = tokio_serial::new(port, 115200).open_native_async()?;
        println!("now connected, start forwarding traffic");
        tokio::io::copy_bidirectional(&mut tcp_stream, &mut serial_steam).await?;
    }

    Ok(())
}

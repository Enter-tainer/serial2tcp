use anyhow::{Ok, Result};
use clap::Parser;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
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
        #[clap(
            required = true,
            help = "name of serial port, e.g. /dev/ttyUSB1 for Linux, COM1 for Windows"
        )]
        serial_port: String,
    },
    #[clap(name = "serve", about = "serve a local serial port to tcp")]
    Serve {
        #[clap(
            required = true,
            help = "name of serial port, e.g. /dev/ttyUSB1 for Linux, COM1 for Windows"
        )]
        serial_port: String,
        #[clap(
            default_value("[::]:23000"),
            help("server ip address and port, e.g. [::]:23000")
        )]
        addr: String,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::List => {
            let t = tokio_serial::available_ports()?;
            for i in t {
                println!("{}", i.port_name);
            }
        }
        Action::Connect { addr, serial_port } => {
            let mut tcp_stream = TcpStream::connect(addr).await?;
            tcp_stream.set_nodelay(true)?;
            let mut serial_steam = tokio_serial::new(serial_port, 115200).open_native_async()?;
            println!("now connected, start forwarding traffic");
            tokio::io::copy_bidirectional(&mut tcp_stream, &mut serial_steam).await?;
        }
        Action::Serve { addr, serial_port } => {
            let tcp_listener = TcpListener::bind(addr).await?;
            loop {
                let (mut socket, addr) = tcp_listener.accept().await?;
                let mut serial_steam =
                    tokio_serial::new(&serial_port, 115200).open_native_async()?;
                println!("now connected to {:#?}, start forwarding traffic", addr);
                tokio::io::copy_bidirectional(&mut socket, &mut serial_steam).await?;
            }
        }
    }
    Ok(())
}

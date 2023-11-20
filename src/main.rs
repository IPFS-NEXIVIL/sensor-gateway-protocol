mod ble;
mod wifi;
use ble::btlescan;
use futures::{pin_mut, FutureExt};
use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use std::{
    io::{ErrorKind, Write},
    sync::Arc,
    time::Duration,
};
use wifi::startWifi;
// use tokio::net::UdpSocket;
use tokio::time;
use tokio::{sync::Notify, time::sleep};
// enum Command {
//     "getPorts"
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cancel = Arc::new(Notify::new());
    let (mut rl, mut stdout) = Readline::new(format!("$"))?;

    let opened_serial: Option<u8>;

    fn get_ports(mut stdout: SharedWriter) {
        let ports = serialport::available_ports().expect("No ports found!");
        for p in ports {
            writeln!(stdout, "{}", p.port_name).unwrap();
        }
    }
    fn open_port(mut stdout: SharedWriter, cancel: Arc<Notify>, name: &str, baudrate: u32) {
        let tstring = [
            "<21020001,REQ,1234>".to_ascii_uppercase(),
            "<22020001,REQ,1234>".to_ascii_uppercase(),
            "<23020001,REQ,1234>".to_ascii_uppercase(),
        ]
        .into_iter()
        .cycle();

        let _name = name.clone().to_owned();
        let port = serialport::new(_name.clone(), baudrate)
            .timeout(Duration::from_millis(700))
            .open()
            .expect("Failed to Open");

        let mut cloned_port1 = port.try_clone().expect("Failed to clone");
        let mut cloned_port2 = port.try_clone().expect("Failed to clone2");
        tokio::spawn(async move {
            let mut _tstring = tstring.clone();

            loop {
                cloned_port2
                    .write_all(_tstring.next().unwrap().as_bytes())
                    .expect("Failed to write to serial port");
                sleep(Duration::from_millis(240)).await;
            }
        });
        tokio::spawn(async move {
            let _ = writeln!(stdout, "{} {}", &_name, &baudrate);

            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", &_name, &baudrate);
            loop {
                match cloned_port1.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let _ = writeln!(stdout, "{}", String::from_utf8_lossy(&serial_buf[..t]));
                    }
                    Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                }
            }
        });
    }

    tokio::task::yield_now().await;

    loop {
        tokio::select! {
            evt = rl.readline().fuse() => match evt {
                Ok(ReadlineEvent::Line(line)) => {
                    let _command:Vec<&str> = line.split_whitespace().collect();

                    match _command[0] {
                        "ports" => {
                            get_ports(stdout.clone());
                        }
                        "open_port" => {
                            if _command.len()<2 {
                                let _=writeln!(stdout, "Error: Arguments");
                            } else {
                                let _port = _command.get(1).unwrap();
                                let _baudrate:u32 = _command.get(2).unwrap_or(&"115200").trim().parse().unwrap();
                                open_port(stdout.clone(), cancel.clone(), _port, _baudrate);
                            }
                        }
                        "ble_central" =>{
                            btlescan();
                        }
                        "wifi" => {
                            startWifi();
                        }
                        _=>{
                            writeln!(stdout, "").unwrap();
                        }
                    }
                }
                Ok(ReadlineEvent::Eof) => {
                    cancel.notify_one();
                    break
                },
                Ok(ReadlineEvent::Interrupted) => {
                    cancel.notify_one();
                    break
                },
                Err(e) => {
                    writeln!(stdout, "Error: {e}")?;
                    writeln!(stdout, "Exiting...")?;
                    break
                },
            }
        }
    }
    Ok(())
}

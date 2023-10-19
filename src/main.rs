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
        let tstring: String = "<21020001,REQ,1234>".to_ascii_uppercase();
        // let tstring2: String = "<22020001,REQ,1234>".to_ascii_uppercase();
        let _name = name.clone().to_owned();
        let port = serialport::new(_name.clone(), baudrate)
            .timeout(Duration::from_millis(700))
            .open()
            .expect("Failed to Open");

        let mut cloned_port1 = port.try_clone().expect("Failed to clone");
        let mut cloned_port2 = port.try_clone().expect("Failed to clone2");
        tokio::spawn(async move {
            let _tstring = tstring.clone();
            // let _tstring2 = tstring2.clone();
            loop {
                cloned_port2
                    .write_all(_tstring.as_bytes())
                    .expect("Failed to write to serial port");
                sleep(Duration::from_millis(700)).await;
                // cloned_port2
                //     .write_all(_tstring2.as_bytes())
                //     .expect("Failed to write to serial port");
                // sleep(Duration::from_millis(350)).await;
            }
        });
        tokio::spawn(async move {
            let _ = writeln!(stdout, "{} {}", &_name, &baudrate);

            // let mut serial_buf: Vec<u8> = vec![0; 1000];

            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", &_name, &baudrate);
            loop {
                // tokio::select! {
                //     _ = cancel.notified() => {
                //         let _ = writeln!(stdout, "Stopped",);

                //         break;
                //     }
                //     else=>{}
                // };
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

            // loop {
            //     tokio::select! {
            //         _ = cancel.notified() => {break},
            //         else => match port.read(serial_buf.as_mut_slice()) {
            //             Ok(t) => {
            //                 let _ = writeln!(stdout, "{}", String::from_utf8_lossy(&serial_buf[..t]));
            //             }
            //             Err(ref e) if e.kind() == ErrorKind::TimedOut => {
            //                 // port.write(_tstring.as_bytes()).unwrap();
            //                 // tokio::time::sleep(time::Duration::from_secs(2)).await;
            //                 let _ = writeln!(stdout, "TEST");

            //             },
            //             Err(e) => eprintln!("{:?}", e),
            //         }
            //     }
            // }
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

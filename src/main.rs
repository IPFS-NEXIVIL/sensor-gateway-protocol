mod ble;

use ble::btlescan;
use futures::{pin_mut, FutureExt};
use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use std::{
    io::{ErrorKind, Write},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Notify;
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
        let _name = name.clone().to_owned();
        tokio::spawn(async move {
            let mut port = serialport::new(_name, baudrate)
                .timeout(Duration::from_millis(10))
                .open()
                .unwrap();
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            loop {
                tokio::select! {
                    _ = cancel.notified() => {break},
                    else => match port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {
                            let _ = writeln!(stdout, "{}", String::from_utf8_lossy(&serial_buf[..t]));
                        }
                        Err(ref e) if e.kind() == ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
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

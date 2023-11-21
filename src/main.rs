mod ble;
mod wifi;
// use ble::btlescan;
use chrono::Utc;
// use futures::FutureExt;
// use rustyline_async::{Readline, ReadlineEvent, SharedWriter};
use std::net::SocketAddr;
use std::{
    io::{ErrorKind, Write},
    sync::Arc,
    time::Duration,
};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;
// use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, Notify};
use tokio::time::sleep;
// use wifi::startWifi;
use tokio::signal;

use clap::Parser;
use regex::Regex;
// use rand::{rngs::StdRng, Rng, SeedableRng};

// enum Command {
//     "getPorts"
// }

#[derive(Debug, Parser)]
#[clap(name = "test-serial")]
struct Opt {
    #[clap(long)]
    port: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let listener = UdpSocket::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).await?;
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let noti = Arc::new(Notify::new());
    let (tx_done, mut rx_done) = mpsc::unbounded_channel::<String>();

    tokio::spawn({
        let mut path: String = format!(
            "./shared/test-{}.txt",
            Utc::now().format("%Y_%d_%m-%H_%M_%S")
        );

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())
            .await?;

        let cloned_tx = tx_done.clone();
        let cloned_noti = noti.clone();
        let re = Regex::new(r"<(?<id>\d{2}).+><(?<sensors>.*)>(?<captime>.*)").unwrap();

        async move {
            loop {
                tokio::select! {
                    Some(_data) = rx.recv() =>{
                        if let Some(caps)=re.captures(&_data) {
                            // println!("{}",&caps["sensors"]);
                            file.write_all(format!("{} {} {}\n",&caps["id"],&caps["sensors"].split(",").collect::<Vec<&str>>()[2..8].join(" "),&caps["captime"]).as_bytes()).await.unwrap();

                        }
                    }
                    _=cloned_noti.notified()=>{
                        // let _path=path.clone();
                        let _=cloned_tx.send(path.clone());
                        path=format!("./shared/test-{}.txt", Utc::now().format("%Y_%d_%m-%H_%M_%S"));
                        file=OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(path.clone())
                        .await.unwrap();
                        // break
                    }
                }
            }
        }
    });
    tokio::spawn({
        let cloned_noti = noti.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3)).await;
                cloned_noti.notify_one();
            }
        }
    });

    // tokio::spawn({
    //     let cloned_tx = tx.clone();
    //     // let cloned_tx2 = tx.clone();
    //     // let mut rng = rand::thread_rng();

    //     let mut rng = {
    //         let rng = rand::thread_rng();
    //         StdRng::from_rng(rng).unwrap()
    //     };

    //     async move {
    //         let mut i = 0;
    //         let sleep_duration = 1000 / 200;
    //         loop {
    //             let _ = cloned_tx.send(format!(
    //                 "{}  {}\n",
    //                 rng.gen::<f64>().to_string(),
    //                 Utc::now().timestamp_millis()
    //             ));
    //             i = i + 1;
    //             tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
    //         }
    //     }
    // });

    let mut tstring = [
        "<21020001,REQ,1234>".to_ascii_uppercase(),
        "<22020001,REQ,1234>".to_ascii_uppercase(),
        // "<23020001,REQ,1234>".to_ascii_uppercase(),
    ]
    .into_iter()
    .cycle();

    let _name = opt.port;
    let baudrate: u32 = "115200".parse().unwrap();
    let port = serialport::new(_name.clone(), baudrate)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(340))
        .open()
        .expect("Failed to Open");

    let mut cloned_port1 = port.try_clone().expect("Failed to clone");
    // let mut cloned_port2 = port.try_clone().expect("Failed to clone2");

    tokio::spawn({
        let cloned_tx = tx.clone();
        let mut _tstring = tstring.clone();

        async move {
            let mut serial_buf: Vec<u8> = vec![0; 1024];
            println!("Receiving data on {} at {} baud:", &_name, &baudrate);
            cloned_port1
                .write_all(tstring.next().unwrap().as_bytes())
                .expect("");
            loop {
                match cloned_port1.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let _ = cloned_tx.send(format!(
                            "{}{}\n",
                            String::from_utf8_lossy(&serial_buf[..t]),
                            Utc::now().timestamp_millis()
                        ));

                        // sleep(Duration::from_millis(100)).await;

                        cloned_port1
                            .write_all(_tstring.next().unwrap().as_bytes())
                            .expect("");
                    }
                    Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                    Err(e) => {
                        eprintln!("{:?}", e);
                        break;
                    }
                }
                sleep(Duration::from_millis(10)).await;
            }
        }
    });

    // tokio::spawn({
    //     let mut _tstring = tstring.clone();
    //     async move {
    //         loop {
    //             cloned_port2
    //                 .write_all(_tstring.next().unwrap().as_bytes())
    //                 .expect("Failed to write to serial port");
    //             sleep(Duration::from_millis(200)).await;
    //         }
    //     }
    // });

    tokio::spawn({
        async move {
            loop {
                tokio::select! {
                Some(_path) = rx_done.recv()=>{
                    // println!("{}",_path);
                    let _ = listener.send_to(
                            &_path.as_bytes(),
                            "127.0.0.1:3132".parse::<SocketAddr>().unwrap(),
                        ).await.unwrap();
                    }
                }
            }
        }
    });

    tokio::task::yield_now().await;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("SHUTDOWN");
                // cancel_token.notify_one();
                break
            }
        }
    }
    Ok(())
}

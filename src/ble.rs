// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::api::{CharPropFlags, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

async fn find_device(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| {
                println!("{}", name);
                name.contains("HC-42-22")
            })
        {
            return Some(p);
        }
    }
    None
}

pub fn btlescan() {
    tokio::spawn({
        async move {
            let manager = Manager::new().await.unwrap();

            // get the first bluetooth adapter
            // connect to the adapter
            let central = get_central(&manager).await;
            // start scanning for devices
            central.start_scan(ScanFilter::default()).await.unwrap();
            // instead of waiting, you can use central.events() to get a stream which will
            // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            let light = find_device(&central).await.unwrap();
            light.connect().await.unwrap();

            // discover services and characteristics
            light.discover_services().await.unwrap();

            // find the characteristic we want
            let chars = light.characteristics();
            // chars.iter().for_each(|c| println!("{}", c));
            let cmd_char = chars
                .iter()
                .find(|c| c.properties.contains(CharPropFlags::NOTIFY))
                .unwrap();
            light.subscribe(&cmd_char).await.unwrap();
            let mut notification_stream = light.notifications().await.unwrap().take(1);
            light
                .write(
                    &cmd_char,
                    b"<22020001,REQ,1234>",
                    btleplug::api::WriteType::WithResponse,
                )
                .await
                .unwrap();
            while let Some(data) = notification_stream.next().await {
                println!("Received data from {:?}", String::from_utf8(data.value));
            }
            // Each adapter has an event stream, we fetch via events(),
            // simplifying the type, this will return what is essentially a
            // Future<Result<Stream<Item=CentralEvent>>>.
            // let mut events = central.events().await.unwrap();

            // start scanning for devices
            // central.start_scan(ScanFilter::default()).await.unwrap();

            // // Print based on whatever the event receiver outputs. Note that the event
            // // receiver blocks, so in a real program, this should be run in its own
            // // thread (not task, as this library does not yet use async channels).
            // while let Some(event) = events.next().await {
            //     match event {
            //         CentralEvent::DeviceDiscovered(id) => {
            //             if format!("{:?}", &id).starts_with("C9") {
            //                 println!("DeviceDiscovered: {:?}", id);
            //             };
            //         }
            //         // CentralEvent::DeviceConnected(id) => {
            //         //     println!("DeviceConnected: {:?}", id);
            //         // }
            //         // CentralEvent::DeviceDisconnected(id) => {
            //         //     println!("DeviceDisconnected: {:?}", id);
            //         // }
            //         // CentralEvent::ManufacturerDataAdvertisement {
            //         //     id,
            //         //     manufacturer_data,
            //         // } => {
            //         //     println!(
            //         //         "ManufacturerDataAdvertisement: {:?}, {:?}",
            //         //         id, manufacturer_data
            //         //     );
            //         // }
            //         // CentralEvent::ServiceDataAdvertisement { id, service_data } => {
            //         //     println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
            //         // }
            //         // CentralEvent::ServicesAdvertisement { id, services } => {
            //         //     let services: Vec<String> =
            //         //         services.into_iter().map(|s| s.to_short_string()).collect();
            //         //     println!("ServicesAdvertisement: {:?}, {:?}", id, services);
            //         // }
            //         _ => {}
            //     }
            // }
        }
    });
}

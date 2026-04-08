#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i64, rx_speed)]
        #[qproperty(i64, tx_speed)]
        type NetworkMonitor = super::NetworkMonitorRust;

        #[qinvokable]
        fn update(self: Pin<&mut Self>);
    }
}

use core::pin::Pin;
use std::sync::{Mutex, OnceLock};
use sysinfo::Networks;

// sysinfo Networks tracks deltas internally — store it across updates
static NETWORKS: OnceLock<Mutex<Networks>> = OnceLock::new();

#[derive(Default)]
pub struct NetworkMonitorRust {
    rx_speed: i64,
    tx_speed: i64,
}

impl qobject::NetworkMonitor {
    fn update(mut self: Pin<&mut Self>) {
        let networks = NETWORKS.get_or_init(|| Mutex::new(Networks::new_with_refreshed_list()));
        let mut networks = networks.lock().unwrap();
        networks.refresh(true);

        let iface = default_iface();

        let (rx, tx) = networks
            .iter()
            .find(|(name, _)| **name == iface)
            .map(|(_, data)| (data.received() as i64, data.transmitted() as i64))
            .unwrap_or((0, 0));

        self.as_mut().set_rx_speed(rx);
        self.as_mut().set_tx_speed(tx);
    }
}

fn default_iface() -> String {
    std::fs::read_to_string("/proc/net/route")
        .ok()
        .and_then(|content| {
            content.lines().skip(1).find_map(|line| {
                let mut parts = line.split_whitespace();
                let iface = parts.next()?;
                let dest = parts.next()?;
                (dest == "00000000").then(|| iface.to_string())
            })
        })
        .unwrap_or_else(|| "eth0".to_string())
}

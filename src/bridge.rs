#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(u64, rx_speed)]
        #[qproperty(u64, tx_speed)]
        type NetworkMonitor = super::NetworkMonitorRust;

        #[qinvokable]
        fn update(self: Pin<&mut Self>);
    }
}

use core::pin::Pin;
use std::cell::RefCell;
use sysinfo::Networks;

#[derive(Default)]
pub struct NetworkMonitorRust {
    rx_speed: u64,
    tx_speed: u64,
    networks: RefCell<Networks>,
}

impl qobject::NetworkMonitor {
    fn update(mut self: Pin<&mut Self>) {
        let iface = default_iface();

        let (rx, tx) = {
            let mut networks = self.networks.borrow_mut();
            networks.refresh(true);
            networks
                .iter()
                .find(|(name, _)| **name == iface)
                .map(|(_, data)| (data.received(), data.transmitted()))
                .unwrap_or((0, 0))
        };

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

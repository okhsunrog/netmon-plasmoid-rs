#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(u64, rx_speed)]
        #[qproperty(u64, tx_speed)]
        #[qproperty(QString, error)]
        type NetworkMonitor = super::NetworkMonitorRust;

        #[qinvokable]
        fn update(self: Pin<&mut Self>);
    }
}

use core::pin::Pin;
use std::cell::RefCell;
use cxx_qt_lib::QString;
use sysinfo::Networks;

#[derive(Default)]
pub struct NetworkMonitorRust {
    rx_speed: u64,
    tx_speed: u64,
    error: QString,
    networks: RefCell<Networks>,
}

impl qobject::NetworkMonitor {
    fn update(mut self: Pin<&mut Self>) {
        let iface = default_iface();

        let (rx, tx, err) = {
            // SAFETY: cxx-qt guarantees we can safely access our struct
            let this = unsafe { self.as_mut().get_unchecked_mut() };

            match default_iface().and_then(|iface| {
                let mut networks = this.networks.borrow_mut();
                networks.refresh(true);

                networks
                    .iter()
                    .find(|(name, _)| **name == iface)
                    .map(|(_, data)| (data.received(), data.transmitted()))
                    .ok_or_else(|| format!("Interface '{}' not found", iface))
            }) {
                Ok((rx, tx)) => (rx, tx, String::new()),
                Err(e) => (0, 0, e),
            }
        };

        self.as_mut().set_rx_speed(rx);
        self.as_mut().set_tx_speed(tx);
        self.as_mut().set_error(QString::from(&err));
    }
}

fn default_iface() -> Result<String, String> {
    let route_content = std::fs::read_to_string("/proc/net/route")
        .map_err(|e| format!("Cannot read /proc/net/route: {}", e))?;

    route_content
        .lines()
        .skip(1)
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let iface = parts.next()?;
            let dest = parts.next()?;
            (dest == "00000000").then(|| iface.to_string())
        })
        .ok_or_else(|| "No default route found in /proc/net/route".to_string())
}

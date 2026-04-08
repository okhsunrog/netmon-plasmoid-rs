#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, rx_speed)]
        #[qproperty(f64, tx_speed)]
        #[qproperty(QString, iface)]
        #[qproperty(bool, link_up)]
        #[qproperty(QString, error)]
        type NetworkMonitor = super::NetworkMonitorRust;

        #[qinvokable]
        fn start(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for NetworkMonitor {}
}

use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use std::io::Read;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct NetworkMonitorRust {
    rx_speed: f64,
    tx_speed: f64,
    iface: QString,
    link_up: bool,
    error: QString,
    shutdown: Option<Sender<()>>,
}

impl Drop for NetworkMonitorRust {
    fn drop(&mut self) {
        // Dropping the sender causes the worker's recv_timeout to return
        // Disconnected, which breaks the loop cleanly.
        self.shutdown.take();
    }
}

impl qobject::NetworkMonitor {
    fn start(mut self: Pin<&mut Self>) {
        if self.as_mut().rust().shutdown.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel::<()>();
        let qt_thread = self.as_mut().qt_thread();
        self.as_mut().rust_mut().get_mut().shutdown = Some(tx);
        std::thread::spawn(move || worker(qt_thread, rx));
    }
}

fn worker(qt_thread: cxx_qt::CxxQtThread<qobject::NetworkMonitor>, shutdown: mpsc::Receiver<()>) {
    const INTERVAL: Duration = Duration::from_millis(1000);
    const ALPHA: f64 = 0.4; // EMA smoothing factor

    let mut proc_buf = String::with_capacity(4096);
    let mut baseline: Option<(u64, u64, Instant)> = None;
    let mut smoothed_rx = 0.0f64;
    let mut smoothed_tx = 0.0f64;
    let mut current_iface: Option<String> = None;

    loop {
        let tick_start = Instant::now();

        let (rx_bps, tx_bps, iface_opt, err) = match default_iface()
            .and_then(|i| read_iface_bytes(&i, &mut proc_buf).map(|b| (i, b)))
        {
            Ok((iface, (rx_bytes, tx_bytes))) => {
                // Reset baseline on iface change.
                if current_iface.as_deref() != Some(iface.as_str()) {
                    baseline = None;
                    current_iface = Some(iface.clone());
                }

                let now = Instant::now();
                let (rx_bps, tx_bps) = match baseline {
                    Some((last_rx, last_tx, last_t)) => {
                        let dt = now.duration_since(last_t).as_secs_f64();
                        // Guard against counter decrease (iface reset, wraparound)
                        // and zero dt. Reset baseline, emit 0 for this tick.
                        if dt > 0.0 && rx_bytes >= last_rx && tx_bytes >= last_tx {
                            (
                                (rx_bytes - last_rx) as f64 / dt,
                                (tx_bytes - last_tx) as f64 / dt,
                            )
                        } else {
                            (0.0, 0.0)
                        }
                    }
                    None => (0.0, 0.0),
                };
                baseline = Some((rx_bytes, tx_bytes, now));
                (rx_bps, tx_bps, Some(iface), String::new())
            }
            Err(e) => {
                // No default route or iface vanished — drop baseline so a
                // reconnect starts fresh instead of spiking.
                baseline = None;
                current_iface = None;
                smoothed_rx = 0.0;
                smoothed_tx = 0.0;
                (0.0, 0.0, None, e)
            }
        };

        smoothed_rx = ALPHA * rx_bps + (1.0 - ALPHA) * smoothed_rx;
        smoothed_tx = ALPHA * tx_bps + (1.0 - ALPHA) * smoothed_tx;

        // Snapshot for the closure (f64 is Copy; strings need clone).
        let rx_out = smoothed_rx;
        let tx_out = smoothed_tx;
        let link_up = iface_opt.is_some();
        let iface_str = iface_opt.unwrap_or_default();
        let err_str = err;

        let _ = qt_thread.queue(move |mut qobj| {
            qobj.as_mut().set_rx_speed(rx_out);
            qobj.as_mut().set_tx_speed(tx_out);
            qobj.as_mut().set_iface(QString::from(&iface_str));
            qobj.as_mut().set_link_up(link_up);
            qobj.as_mut().set_error(QString::from(&err_str));
        });

        // Sleep the remainder of the interval so cadence doesn't drift.
        let elapsed = tick_start.elapsed();
        let remaining = INTERVAL.saturating_sub(elapsed);
        match shutdown.recv_timeout(remaining) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

fn default_iface() -> Result<String, String> {
    let route = std::fs::read_to_string("/proc/net/route")
        .map_err(|e| format!("Cannot read /proc/net/route: {}", e))?;
    route
        .lines()
        .skip(1)
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let iface = parts.next()?;
            let dest = parts.next()?;
            (dest == "00000000").then(|| iface.to_string())
        })
        .ok_or_else(|| "No default route".to_string())
}

fn read_iface_bytes(iface: &str, buf: &mut String) -> Result<(u64, u64), String> {
    buf.clear();
    std::fs::File::open("/proc/net/dev")
        .and_then(|mut f| f.read_to_string(buf))
        .map_err(|e| format!("Cannot read /proc/net/dev: {}", e))?;

    for line in buf.lines() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        if line[..colon].trim() != iface {
            continue;
        }
        let cols: Vec<&str> = line[colon + 1..].split_whitespace().collect();
        // Columns: 0=rx_bytes, 1=rx_packets, ..., 8=tx_bytes
        if cols.len() < 9 {
            return Err(format!("Malformed /proc/net/dev line for {}", iface));
        }
        let rx = cols[0]
            .parse::<u64>()
            .map_err(|e| format!("Parse rx: {}", e))?;
        let tx = cols[8]
            .parse::<u64>()
            .map_err(|e| format!("Parse tx: {}", e))?;
        return Ok((rx, tx));
    }
    Err(format!("Interface '{}' not in /proc/net/dev", iface))
}

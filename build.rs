use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("org.kde.plasma.rustyapplet")
            .version(1, 0)
            .plugin_type(PluginType::Dynamic),
    )
    .qt_module("Quick")
    .file("src/bridge.rs")
    .build();

    // Export Qt plugin entry points as dynamic symbols so plasmashell can load
    // the plugin via dlsym. We use a version script instead of CMake.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg=-Wl,--version-script={manifest}/plugin.version");
}

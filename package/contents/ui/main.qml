import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.components as PlasmaComponents
import org.kde.kirigami as Kirigami
import org.kde.plasma.rustyapplet

PlasmoidItem {
    id: root

    NetworkMonitor {
        id: monitor
        Component.onCompleted: update()
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: monitor.update()
    }

    function formatSpeed(bytes) {
        const bits = bytes * 8
        if (bits < 1000)
            return bits + " bps"
        else if (bits < 1000 * 1000)
            return (bits / 1000).toFixed(1) + " Kbps"
        else
            return (bits / (1000 * 1000)).toFixed(2) + " Mbps"
    }

    // Panel representation: two compact lines
    compactRepresentation: MouseArea {
        onClicked: root.expanded = !root.expanded

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 0

            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignHCenter
                text: "▼ " + formatSpeed(monitor.rx_speed)
                font.pixelSize: 10
                color: Kirigami.Theme.positiveTextColor
            }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignHCenter
                text: "▲ " + formatSpeed(monitor.tx_speed)
                font.pixelSize: 10
                color: Kirigami.Theme.neutralTextColor
            }
        }
    }

    // Popup representation
    fullRepresentation: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing
        implicitWidth: Kirigami.Units.gridUnit * 14

        Kirigami.Heading {
            Layout.alignment: Qt.AlignHCenter
            text: "Network Traffic"
            level: 3
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Kirigami.Units.largeSpacing

            PlasmaComponents.Label { text: "Download:" }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignRight
                text: formatSpeed(monitor.rx_speed)
                color: Kirigami.Theme.positiveTextColor
                font.bold: true
            }

            PlasmaComponents.Label { text: "Upload:" }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignRight
                text: formatSpeed(monitor.tx_speed)
                color: Kirigami.Theme.neutralTextColor
                font.bold: true
            }
        }
    }
}

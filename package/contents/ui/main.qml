import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.components as PlasmaComponents
import org.kde.kirigami as Kirigami
import org.kde.plasma.rustyapplet

PlasmoidItem {
    id: root

    readonly property int historySize: 60
    property var rxHistory: Array(historySize).fill(0)
    property var txHistory: Array(historySize).fill(0)

    NetworkMonitor {
        id: monitor
        Component.onCompleted: update()
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            monitor.update()
            rxHistory = [...rxHistory.slice(1), monitor.rx_speed]
            txHistory = [...txHistory.slice(1), monitor.tx_speed]
        }
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
        implicitWidth: Kirigami.Units.gridUnit * 18

        Kirigami.Heading {
            Layout.alignment: Qt.AlignHCenter
            text: "Network Traffic"
            level: 3
        }

        // Scrolling plot
        Canvas {
            id: plot
            Layout.fillWidth: true
            implicitHeight: Kirigami.Units.gridUnit * 8

            onVisibleChanged: if (visible) requestPaint()

            Connections {
                target: root
                function onRxHistoryChanged() { plot.requestPaint() }
            }

            onPaint: {
                const ctx = getContext("2d")
                const w = width, h = height
                const rx = root.rxHistory
                const tx = root.txHistory
                const n = rx.length

                // Background
                ctx.clearRect(0, 0, w, h)
                ctx.fillStyle = Qt.rgba(0, 0, 0, 0.15)
                ctx.fillRect(0, 0, w, h)

                // Scale to max value in view
                let maxVal = 1
                for (let i = 0; i < n; i++) {
                    if (rx[i] * 8 > maxVal) maxVal = rx[i] * 8
                    if (tx[i] * 8 > maxVal) maxVal = tx[i] * 8
                }

                function xPos(i) { return (i / (n - 1)) * w }
                function yPos(v) { return h - (v * 8 / maxVal) * (h * 0.9) - h * 0.05 }

                function drawLine(history, color) {
                    ctx.beginPath()
                    ctx.strokeStyle = color
                    ctx.lineWidth = 1.5
                    ctx.moveTo(xPos(0), yPos(history[0]))
                    for (let i = 1; i < n; i++)
                        ctx.lineTo(xPos(i), yPos(history[i]))
                    ctx.stroke()
                }

                drawLine(rx, Kirigami.Theme.positiveTextColor)
                drawLine(tx, Kirigami.Theme.neutralTextColor)
            }
        }

        // Legend + current values
        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Kirigami.Units.largeSpacing

            RowLayout {
                spacing: Kirigami.Units.smallSpacing
                Rectangle { width: 12; height: 2; color: Kirigami.Theme.positiveTextColor }
                PlasmaComponents.Label { text: "Download:" }
            }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignRight
                text: formatSpeed(monitor.rx_speed)
                color: Kirigami.Theme.positiveTextColor
                font.bold: true
            }

            RowLayout {
                spacing: Kirigami.Units.smallSpacing
                Rectangle { width: 12; height: 2; color: Kirigami.Theme.neutralTextColor }
                PlasmaComponents.Label { text: "Upload:" }
            }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignRight
                text: formatSpeed(monitor.tx_speed)
                color: Kirigami.Theme.neutralTextColor
                font.bold: true
            }
        }
    }
}

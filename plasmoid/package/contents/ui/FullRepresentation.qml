import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.plasma.plasmoid
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: fullRoot

    property int currentStop: 2
    property string autoMode: "manual"
    property string ppdCoupling: "coupled"
    property int currentPpdStop: 2   // 1=power-saver, 2=balanced, 3=performance

    signal stopRequested(int stop)
    signal ppdProfileRequested(string profile)

    readonly property var stopLabels: ["", "Save", "Adaptive", "Standard", "Perf", "Max"]
    readonly property var ppdStopLabels: ["", "Power Saver", "Balanced", "Performance"]
    readonly property var ppdProfiles: ["", "power-saver", "balanced", "performance"]
    readonly property var stopColors: [
        "",
        "#1a5276",
        "#117a65",
        "#b7770d",
        "#d35400",
        "#c0392b"
    ]

    spacing: Kirigami.Units.smallSpacing

    Kirigami.Heading {
        level: 3
        text: "GPU Profile"
        Layout.alignment: Qt.AlignHCenter
    }

    PlasmaComponents.Label {
        text: fullRoot.stopLabels[fullRoot.currentStop] || ""
        font.bold: true
        Layout.alignment: Qt.AlignHCenter
        color: fullRoot.stopColors[fullRoot.currentStop] || Kirigami.Theme.textColor
    }

    QQC2.Slider {
        id: profileSlider
        from: 1
        to: 5
        stepSize: 1
        snapMode: QQC2.Slider.SnapAlways
        value: fullRoot.currentStop
        Layout.fillWidth: true
        Layout.leftMargin: Kirigami.Units.largeSpacing
        Layout.rightMargin: Kirigami.Units.largeSpacing

        onMoved: {
            fullRoot.stopRequested(Math.round(value))
        }
    }

    // Sync slider to external currentStop changes without feedback loop
    Binding {
        target: profileSlider
        property: "value"
        value: fullRoot.currentStop
        when: !profileSlider.pressed
    }

    RowLayout {
        Layout.fillWidth: true
        Layout.leftMargin: Kirigami.Units.largeSpacing
        Layout.rightMargin: Kirigami.Units.largeSpacing

        Repeater {
            model: ["Save", "Adapt", "Std", "Perf", "Max"]
            PlasmaComponents.Label {
                Layout.fillWidth: true
                text: modelData
                horizontalAlignment: Text.AlignHCenter
                font.pixelSize: 10
                opacity: (index + 1 === fullRoot.currentStop) ? 1.0 : 0.5
            }
        }
    }

    Kirigami.Separator {
        Layout.fillWidth: true
    }

    // PPD independent slider — only shown when coupling = independent
    Kirigami.Heading {
        level: 3
        text: "Power Profile"
        Layout.alignment: Qt.AlignHCenter
        visible: fullRoot.ppdCoupling === "independent"
    }

    PlasmaComponents.Label {
        text: fullRoot.ppdStopLabels[fullRoot.currentPpdStop] || ""
        font.bold: true
        Layout.alignment: Qt.AlignHCenter
        visible: fullRoot.ppdCoupling === "independent"
    }

    QQC2.Slider {
        id: ppdSlider
        from: 1
        to: 3
        stepSize: 1
        snapMode: QQC2.Slider.SnapAlways
        value: fullRoot.currentPpdStop
        Layout.fillWidth: true
        Layout.leftMargin: Kirigami.Units.largeSpacing
        Layout.rightMargin: Kirigami.Units.largeSpacing
        visible: fullRoot.ppdCoupling === "independent"

        onMoved: {
            var stop = Math.round(value)
            fullRoot.ppdProfileRequested(fullRoot.ppdProfiles[stop])
        }
    }

    Binding {
        target: ppdSlider
        property: "value"
        value: fullRoot.currentPpdStop
        when: !ppdSlider.pressed
    }

    RowLayout {
        Layout.fillWidth: true
        Layout.leftMargin: Kirigami.Units.largeSpacing
        Layout.rightMargin: Kirigami.Units.largeSpacing
        visible: fullRoot.ppdCoupling === "independent"

        Repeater {
            model: ["Saver", "Balanced", "Perf"]
            PlasmaComponents.Label {
                Layout.fillWidth: true
                text: modelData
                horizontalAlignment: Text.AlignHCenter
                font.pixelSize: 10
                opacity: (index + 1 === fullRoot.currentPpdStop) ? 1.0 : 0.5
            }
        }
    }

    Kirigami.Separator {
        Layout.fillWidth: true
        visible: fullRoot.ppdCoupling === "independent"
    }

    PlasmaComponents.Label {
        text: "PPD: not managed"
        opacity: 0.5
        font.italic: true
        Layout.alignment: Qt.AlignHCenter
        visible: fullRoot.ppdCoupling === "gpu_only"
    }

    PlasmaComponents.Label {
        text: "Mode: " + fullRoot.autoMode
        opacity: 0.7
        Layout.alignment: Qt.AlignHCenter
    }
}

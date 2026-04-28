import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: configRoot

    implicitWidth: formLayout.implicitWidth
    implicitHeight: formLayout.implicitHeight

    property string cfg_autoMode: "manual"
    property alias cfg_batteryThreshold: thresholdSpinBox.value
    property alias cfg_acDefaultStop: acStopSpinBox.value
    property string cfg_ppdCoupling: "coupled"

    readonly property var modeValues: ["manual", "ac_battery", "battery_pct"]
    readonly property var couplingValues: ["coupled", "gpu_only", "independent"]

    Kirigami.FormLayout {
        id: formLayout
        anchors.left: parent.left
        anchors.right: parent.right

        QQC2.ComboBox {
            id: autoModeCombo
            Kirigami.FormData.label: "Auto-switch mode:"
            model: ["Manual", "AC/Battery switch", "Battery % threshold"]

            // Initialise from saved config once loaded
            Component.onCompleted: {
                var idx = configRoot.modeValues.indexOf(cfg_autoMode)
                currentIndex = (idx >= 0) ? idx : 0
            }

            // Only fires on user interaction, not programmatic changes
            onActivated: cfg_autoMode = configRoot.modeValues[currentIndex]
        }

        QQC2.SpinBox {
            id: thresholdSpinBox
            Kirigami.FormData.label: "Battery threshold (%):"
            from: 5
            to: 90
            visible: autoModeCombo.currentIndex === 2
        }

        QQC2.SpinBox {
            id: acStopSpinBox
            Kirigami.FormData.label: "AC default stop (1-5):"
            from: 1
            to: 5
        }

        QQC2.ComboBox {
            id: couplingCombo
            Kirigami.FormData.label: "PPD coupling:"
            model: ["GPU + PPD linked", "GPU only (PPD untouched)", "Independent sliders"]

            Component.onCompleted: {
                var idx = configRoot.couplingValues.indexOf(cfg_ppdCoupling)
                currentIndex = (idx >= 0) ? idx : 0
            }

            onActivated: cfg_ppdCoupling = configRoot.couplingValues[currentIndex]
        }
    }
}

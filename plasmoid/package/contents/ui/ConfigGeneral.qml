import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.kcmutils as KCM

KCM.SimpleKCM {
    id: configRoot

    property string cfg_autoMode: "manual"
    property alias cfg_batteryThreshold: thresholdSpinBox.value
    property alias cfg_acDefaultStop: acStopSpinBox.value

    Kirigami.FormLayout {
        anchors.fill: parent

        QQC2.ComboBox {
            id: autoModeCombo
            Kirigami.FormData.label: i18n("Auto-switch mode:")
            model: ListModel {
                ListElement { text: "Manual";              value: "manual"      }
                ListElement { text: "AC/Battery switch";   value: "ac_battery"  }
                ListElement { text: "Battery % threshold"; value: "battery_pct" }
            }
            textRole: "text"
            valueRole: "value"
            currentIndex: {
                for (var i = 0; i < model.count; i++) {
                    if (model.get(i).value === cfg_autoMode) return i
                }
                return 0
            }
            onCurrentIndexChanged: cfg_autoMode = model.get(currentIndex).value
        }

        QQC2.SpinBox {
            id: thresholdSpinBox
            Kirigami.FormData.label: "Battery threshold (%):"
            from: 5
            to: 90
            value: plasmoid.configuration.batteryThreshold
            visible: autoModeCombo.currentValue === "battery_pct"
        }

        QQC2.SpinBox {
            id: acStopSpinBox
            Kirigami.FormData.label: "AC default stop (1-5):"
            from: 1
            to: 5
            value: plasmoid.configuration.acDefaultStop
        }
    }
}

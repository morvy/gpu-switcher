import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.plasma5support as P5Support
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.kirigami as Kirigami

PlasmoidItem {
    id: root

    property int currentStop: 2
    property bool pollerBusy: false
    property string pollerCurrentCmd: ""

    property string currentPpdProfile: "balanced"
    property int currentPpdStop: {
        switch (currentPpdProfile) {
            case "power-saver": return 1
            case "performance": return 3
            default: return 2
        }
    }
    property bool ppdPollerBusy: false
    property string ppdPollerCurrentCmd: ""
    property bool _couplingInitialized: false

    readonly property var stopIcons: [
        "",
        "gpu-switcher-save",
        "gpu-switcher-adaptive",
        "gpu-switcher-balanced",
        "gpu-switcher-perf",
        "gpu-switcher-max"
    ]

    // Poll daemon for current stop every 3 seconds via busctl
    P5Support.DataSource {
        id: poller
        engine: "executable"
        connectedSources: []

        onNewData: function(source, data) {
            var out = data["stdout"] || ""
            // busctl --json=short returns e.g. {"type":"y","data":2}
            var match = out.match(/"data"\s*:\s*(\d+)/)
            if (match) {
                var stop = parseInt(match[1])
                if (stop >= 1 && stop <= 5) root.currentStop = stop
            }
            pollerTimeout.stop()
            disconnectSource(source)
            root.pollerBusy = false
        }
    }

    Timer {
        id: pollerTimeout
        interval: 8000
        repeat: false
        onTriggered: {
            if (root.pollerCurrentCmd !== "") {
                poller.disconnectSource(root.pollerCurrentCmd)
                root.pollerCurrentCmd = ""
            }
            root.pollerBusy = false
        }
    }

    Timer {
        interval: 3000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (root.pollerBusy) return
            root.pollerBusy = true
            var cmd = "busctl --system --json=short get-property net.gpuswitcher.Manager /net/gpuswitcher/Manager net.gpuswitcher.Manager CurrentStop"
            root.pollerCurrentCmd = cmd
            poller.connectSource(cmd)
            pollerTimeout.restart()
        }
    }

    // Fire-and-forget executor for DBus calls
    P5Support.DataSource {
        id: runner
        engine: "executable"
        connectedSources: []
        onNewData: function(source, data) {
            disconnectSource(source)
        }
    }

    function setStop(stop) {
        if (stop < 1 || stop > 5) return
        var cmd = "busctl --system call net.gpuswitcher.Manager /net/gpuswitcher/Manager net.gpuswitcher.Manager SetStop y " + stop
        runner.connectSource(cmd)
        root.currentStop = stop  // optimistic update
    }

    function setPpdProfile(profile) {
        var cmd = "busctl --system call net.gpuswitcher.Manager /net/gpuswitcher/Manager net.gpuswitcher.Manager SetPpdProfile s " + profile
        runner.disconnectSource(cmd)
        runner.connectSource(cmd)
        root.currentPpdProfile = profile  // optimistic update
    }

    // PPD profile poller — only active in independent coupling mode
    P5Support.DataSource {
        id: ppdPoller
        engine: "executable"
        connectedSources: []

        onNewData: function(source, data) {
            var out = data["stdout"] || ""
            var match = out.match(/"data"\s*:\s*"([^"]+)"/)
            if (match) root.currentPpdProfile = match[1]
            ppdPollerTimeout.stop()
            disconnectSource(source)
            root.ppdPollerBusy = false
        }
    }

    Timer {
        id: ppdPollerTimeout
        interval: 8000
        repeat: false
        onTriggered: {
            if (root.ppdPollerCurrentCmd !== "") {
                ppdPoller.disconnectSource(root.ppdPollerCurrentCmd)
                root.ppdPollerCurrentCmd = ""
            }
            root.ppdPollerBusy = false
        }
    }

    Timer {
        interval: 3000
        running: (plasmoid.configuration.ppdCoupling || "coupled") === "independent"
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (root.ppdPollerBusy) return
            root.ppdPollerBusy = true
            var cmd = "busctl --system --json=short get-property net.gpuswitcher.Manager /net/gpuswitcher/Manager net.gpuswitcher.Manager CurrentPpdProfile"
            root.ppdPollerCurrentCmd = cmd
            ppdPoller.connectSource(cmd)
            ppdPollerTimeout.restart()
        }
    }

    // Sync coupling to daemon when plasmoid config changes
    Connections {
        target: plasmoid.configuration
        function onPpdCouplingChanged() {
            if (!root._couplingInitialized) return
            var coupling = plasmoid.configuration.ppdCoupling || "coupled"
            var cmd = "busctl --system call net.gpuswitcher.Manager /net/gpuswitcher/Manager net.gpuswitcher.Manager SetCoupling s " + coupling
            runner.disconnectSource(cmd)
            runner.connectSource(cmd)
        }
    }

    Component.onCompleted: {
        root._couplingInitialized = true
    }

    compactRepresentation: Item {
        width: Kirigami.Units.iconSizes.medium
        height: width

        Kirigami.Icon {
            anchors.fill: parent
            source: root.stopIcons[root.currentStop] || "gpu-switcher-balanced"
            active: mouseArea.containsMouse
        }

        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: root.expanded = !root.expanded
        }
    }

    fullRepresentation: FullRepresentation {
        implicitWidth: Kirigami.Units.gridUnit * 18
        implicitHeight: (plasmoid.configuration.ppdCoupling === "independent")
            ? Kirigami.Units.gridUnit * 22
            : Kirigami.Units.gridUnit * 14
        currentStop: root.currentStop
        autoMode: plasmoid.configuration.autoMode || "manual"
        ppdCoupling: plasmoid.configuration.ppdCoupling || "coupled"
        currentPpdStop: root.currentPpdStop
        onStopRequested: function(stop) { root.setStop(stop) }
        onPpdProfileRequested: function(profile) { root.setPpdProfile(profile) }
    }
}

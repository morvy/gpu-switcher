---
name: Bug report
about: Something isn't working
labels: bug
---

**Describe the bug**


**System info**
```
uname -r:
plasmashell --version:
lspci -nn | grep VGA:
cat /sys/class/drm/card*/device/power_dpm_force_performance_level:
```

**Daemon status**
```
systemctl status gpu-switcher
journalctl -u gpu-switcher -n 30
```

**Expected behavior**


**Steps to reproduce**

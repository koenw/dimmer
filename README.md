# dimmer

Dimmer smoothly transitions your screen from one brightness to another. Tested
only with Wayland and recent Linux kernels.

```sh
# Dim the screen over 3 seconds
dimmer --duration 3s

# Dim the screen, first saving the current brightness to a statefile
dimmer --save --duration 5s

# Restore the screen from a previously saved statefile
dimmer --restore --duration 5s
```

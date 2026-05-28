# dimmer

Dimmer smoothly changes the brightness of your screen.


## Features

* Smooth, configurable transitions
* No special permissions required (using the default [systemd-logind D-Bus backend](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html))
* Writing directly to sysfs is also supported (e.g. if you don't want the
  systemd-logind dependency)
* Brightness can be changed relative to the current brightness, making
  multiple transitions in a row extra satisfying.
* Save/Restore brightness from a file


## Usage

```sh
# Show usage information
dimmer --help

# Show current brightness
dimmer

# Increase the brightness by 10%
dimmer +10%

# Decrease the brightness by 10%
dimmer -10%

# Set to maximum brightness
dimmer 100%

# Save the current brightness to a file
dimmer --save

# Dim (or brighten) the screen to 30%, first saving the current brightness
dimmer --save 30%

# Restore the screen from a previously saved brightness, taking half a second
dimmer --restore --duration 0.5s
```


### Tips for binding to a hotkey

The default value for the transition duration is tweaked for manual use from
the command-line. When binding `dimmer` to a hotkey, I would suggest starting
with a lower duration and a relatively small change, e.g.

```sh
dimmer --duration 100ms +15% # Bound to XF86MonBrightnessUp
dimmer --duration 100ms -15% # Bound to XF86MonBrightnessDown
```


## Installation

<details>
  <summary>`nix run github:koenw/dimmer`</summary>
  Use as you would any nix flake, e.g. run directly with

  ```sh
  nix run github:koenw/dimmer
  ```
</details>

<details>
  <summary>`cargo install dimmer`</summary>
  Install to cargo's bin directory with

  ```sh
  cargo install dimmer
  ```
</details>


### Integration with swayidle

Many people like to automatically turnoff & lock their screen after a period of
idleness and it would sure be nice if we could smoothly dim the screen before
turning it off (and smoothly restore the brightness later). When we interact
with the system during the dimming process, we'd like the dimming process to
stop and the brightness to quickly be restored.

Swayidle offers no possibilities for signals to be send to a (dimming) process
that is already running, so the reaction to the new user input would have to
come from a new process. However, the
[swayidle(1)](https://github.com/swaywm/swayidle/blob/master/swayidle.1.scd)
man-page recommends for security purposes to use the `-w` flag to wait for triggered
commands to finish when used in combination with a screenlock, which would make
it impossible for us to interrupt the screen dimming process on user input.

For this reason, I recommend running one swayidle process *with* the `-w` flag
to trigger the locking command, and another swayidle process *without* the `-w`
flag to trigger the `dimmer` commands.

<details>
  <summary>
    Sway and swayidle configuration example
  </summary>

  Excerpt from what a sway config (e.g. `~/.config/sway/config`) could look
  like:
  ```sway-config
  exec swayidle -w \
    timeout 620 'swaymsg "output * dpms off"' \
    timeout 625 'swaylock -f' \
    timeout 630 'systemctl suspend'

  exec swayidle \
    timeout 600 'dimmer --save' \
    resume 'pkill dimmer; dimmer --restore'
  ```
</details>


### A note on permissions when using the Sysfs backend

**Note that the default D-Bus backend requires no special permissions.**

The Sysfs backend changes the brightness by writing to a special
file in `/sys` (exposed by the kernel for this purpose), which on
many distro's requires the user to be a member of a group, often
`video`. If you receive a *Permission denied* error, chances are
your user needs additional privileges to be able to write to the
file.

Check `ls -l /sys/class/backlight/*/brightness` for the permissions on the
backlight file and the group you can add your user to.

If you're going this route, you'll probably want to write a udev rule to grant
your user the required permissions, but that's behind the scope of this README.

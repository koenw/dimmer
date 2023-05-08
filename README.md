# dimmer

Dimmer smoothly transitions your screen from one brightness to another. Very
simple, and only tested with Wayland and recent Linux kernels.


## Usage

```sh
# Show usage information
dimmer --help

# Dim the screen to zero brightness over 5 seconds
dimmer

# Dim (or brighten) the screen to 30%, first saving the current brightness to a statefile
dimmer --save --duration 5s 30%

# Restore the screen from a previously saved brightness, using 2 seconds
dimmer --restore --duration 2s
```

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
    resume 'pkill dimmer; dimmer --restore --duration 1s'
  ```
</details>


## Installation

### Permissions

Setting the backlight works by writing to a special file in `/sys` (exposed by
the kernel for this purpose), which on many distro's requires the user to be a
member of a group, often `video`. If you receive a *Permission denied* error,
chances are your user needs additional privileges to be able to write to the
file.

Check `ls -l /sys/class/backlight/*/brightness` for the permissions on the
backlight file and the group you can add your user to.

<details>
  <summary>Nix Flakes</summary>
  Use as you would any nix flake, e.g. run directly with

  ```sh
  nix run github:koenw/dimmer
  ```
</details>

<details>
  <summary>Cargo</summary>
  Install to cargo's bin directory with

  ```sh
  cargo install dimmer
  ```
</details>

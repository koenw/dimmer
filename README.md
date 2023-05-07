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
dimmer --save --target 30% --duration 5s

# Restore the screen from a previously saved brightness, using 2 seconds
dimmer --restore --duration 2s
```

## Installation

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

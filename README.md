# Maikor

>⚠️ Work in progress
>
> Links may be broken, features missing, etc

*Cross platform 16 bit game system*

See more at [maikor.app](https://maikor.app)

### Play

[Android](https://github.com/MaikorAppPublic/android-app)

[iOS](https://github.com/MaikorAppPublic/ios-app)

[Windows, macOS and Linux](https://github.com/MaikorAppPublic/desktop-app)

### Make

[iOS IDE](https://github.com/MaikorAppPublic/ios-app)

[Desktop IDE](https://github.com/MaikorAppPublic/desktop-ide)

[Build tools](https://github.com/MaikorAppPublic/build-tools)

[REPL](https://play.vm.maikor.app)

### Project breakdown

#### Major
* [vm-core](https://github.com/MaikorAppPublic/vm-core)
  * Executes Maikor games
* [desktop-app](https://github.com/MaikorAppPublic/desktop-app)
  * Host program for Windows, macOS and Linux
* [android-app](https://github.com/MaikorAppPublic/android-app)
  * Host program for Android
* [ios-app](https://github.com/MaikorAppPublic/ios-app)
  * Host program for iOS
* [desktop-ide](https://github.com/MaikorAppPublic/desktop-ide)
  * IDE program for Windows, macOS and Linux
* [web-repl](https://github.com/MaikorAppPublic/web-repl)
  * Web based REPL

#### Minor
* [vm-interface](https://github.com/MaikorAppPublic/vm-interface)
  * Acts as hardware simulation layer for the VM
* [vm-interface-android](https://github.com/MaikorAppPublic/vm-interface-android)
  * Android compatible wrapper for `vm-interface`
* [vm-interface-ios](https://github.com/MaikorAppPublic/vm-interface-ios)
  * iOS compatible wrapper for `vm-interface`
* [vm-desktop-simple](https://github.com/MaikorAppPublic/vm-desktop-simple)
  * Simple desktop program for testing Maikor games (it can't save, etc)
* [vm-file](https://github.com/MaikorAppPublic/vm-file)
  * For reading and writing Maikor game files
* [maikor-language](https://github.com/MaikorAppPublic/language)
  * OP names, memory addresses and platform requirements
* [vm-interface-wasm](https://github.com/MaikorAppPublic/vm-interface-wasm)
  * WASM compatible wrapper for `vm-interface`

## vm-interface

This provides an interface to use the VM from host apps.
It reads directly from the VM memory to create frames and play sounds, and converts inputs and writes them to memory. It also triggers relevant interrupts.

It essentially simulates hardware for the VM.
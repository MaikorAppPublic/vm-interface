# Maikor

*Cross platform 16 bit game system*

See more at [maikor.app](https://maikor.app)

### Play

[Android](https://github.com/MaikorAppPublic/android-app)

[iOS](https://github.com/MaikorAppPublic/ios-app)

[Windows, macOS and Linux](https://github.com/MaikorAppPublic/desktop-app)

### Make

iOS IDE

Desktop IDE

Build tools

### Project breakdown

#### Major
* [vm-core](https://github.com/MaikorAppPublic/vm-core)
  * Executes Maikor games
* [vm-interface](https://github.com/MaikorAppPublic/vm-interface)
  * Acts as hardware simulation layer for the VM 
* [desktop-app](https://github.com/MaikorAppPublic/desktop-app)
  * Host program for Windows, macOS and Linux
* [android-app](https://github.com/MaikorAppPublic/android-app)
  * Host program for Android
* [ios-app](https://github.com/MaikorAppPublic/ios-app)
  * Host program for iOS

#### Minor
* [vm-interface-android](https://github.com/MaikorAppPublic/vm-interface-android)
  * Android compatible wrapper for `vm-interface`
* [vm-interface-ios](https://github.com/MaikorAppPublic/vm-interface-ios)
  * iOS compatible wrapper for `vm-interface`
* [vm-desktop-simple](https://github.com/MaikorAppPublic/vm-desktop-simple)
  * Simple desktop program for testing Maikor games (it can't save, etc)
* [vm-file](https://github.com/MaikorAppPublic/vm-file) 
  * For reading and writing Maikor game files 
 

## vm-interface

This provides an interface to use the VM from host apps.
It reads directly from the VM memory to create frames and play sounds, and converts inputs and writes them to memory. It also triggers relevant interrupts.

It essentially simulates hardware for the VM.
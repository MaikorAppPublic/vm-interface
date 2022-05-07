# Maikor

>⚠️ Work in progress
>
> Links may be broken, features missing, etc

*Cross platform 16 bit game system*

See more at [maikor.app](https://maikor.app) and the [project homepage](https://github.com/MaikorAppPublic)

### Play

[Android](https://github.com/MaikorAppPublic/android-app)

[iOS](https://github.com/MaikorAppPublic/ios-app)

[Windows, macOS and Linux](https://github.com/MaikorAppPublic/desktop-app)

### Make

[iOS IDE](https://github.com/MaikorAppPublic/ios-app)

[Desktop IDE](https://github.com/MaikorAppPublic/desktop-ide)

[Build tools](https://github.com/MaikorAppPublic/build-tools)

[REPL](https://play.vm.maikor.app)
## vm-interface

This provides an interface to use the VM from host apps.
It reads directly from the VM memory to create frames and play sounds, and converts inputs and writes them to memory. It also triggers relevant interrupts.

It essentially simulates hardware for the VM.
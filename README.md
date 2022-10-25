# mtxcli

Matrix Command Line Interface

![mtxcli](docs/mtxcli-640x120.png)

## Status

This is a revised version of the mtxcli Matrix chat program.
This version, 0.5.0 (and beyond), strives to be as simple as possible
in order to prepare for running on the Xous operating system on
the Betrusted (Precursor) hardware device.

The previous work used the **tokio** library for asynchronous
communication, but became very complex (_NOTE:_ that code
is still available in the `mtxcli-tokio` branch).

This version will display any recent messages after you
type a message to a given room. In this way the code flow
is much easier to develop and debug for the Betrusted environment.

## Prerequsites

Before using this program you should:

1. Setup a new user (if needed) using the Matrix protocol
  * https://matrix.org/docs/projects/try-matrix-now/
2. Create a new room (if needed)
  * https://doc.matrix.tu-dresden.de/en/rooms/create/
3. Join the room

## Usage

When you start using mtxcli you will need to set your user identifer,
your password and the room you would like to join.

You can do this with commands that start with slash `/` (see the example below).
_NOTE_: Type `/help` for a list of available commands.

Perhaps the easist way to explain using mtxcli is by example:
(here I just typed **Hello!** to send a message to the room)

```
tmarble@espoir 996 :) cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/mtxcli`
/set user @info9net:matrix.org
/set password my-secret-password
/set room #toms-brewpub
Hello!
mtxcli> logging in...
mtxcli> logged in
tmarble> This is just a test room...
tmarble> to experiment with matrix chat
tmarble> Please join us!
info9net> Hello!
This is a test
info9net> Hello!
tmarble> Welcome info9net !
info9net> This is a test
Thank you!
info9net> Thank you!
/quit
tmarble@espoir 997 :)

```

## Contribution Guidelines

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.0%20adopted-ff69b4.svg)](CODE_OF_CONDUCT.md)

Please see [CONTRIBUTING](CONTRIBUTING.md) for details on
how to make a contribution.

Please note that this project is released with a
[Contributor Code of Conduct](CODE_OF_CONDUCT.md).
By participating in this project you agree to abide its terms.

## License

Copyright © 2020-2022

Licensed under the [GPL-3.0](https://opensource.org/licenses/GPL-3.0) [LICENSE](LICENSE)

## Acknowledgements

This project is supported in part by a grant from the
[NGI0 PET Fund](https://nlnet.nl/PET/),
a fund established by NLnet with financial support from the
European Commission's [Next Generation Internet](https://www.ngi.eu/) program.

![nl.net+NGI0](https://www.crowdsupply.com/img/001b/precursor-grant-logos_png_md-xl.jpg)

MIDI-to-Keypress
================

Takes MIDI events and turns them into keypresses.  Mostly designed for "Perform" in FFXIV.

See [Releases](https://github.com/xobs/midi-to-keypress/releases) for a list of releases.

Installing
----------

You only need to download the [latest release](https://github.com/xobs/midi-to-keypress/releases/latest).  Download it, unzip it, and run it from cmd.exe.

Building
--------

This program requires Rust.  Download it from [rustup.rs](https://rustup.rs).

To and run, go into this directory and type:

````
cargo run
````

Usage
-----

To list available devices, run "miditran --list".  To specify a device to use as an input, run "miditran --device [device-name]".

Currently, there is no external configuration.  The program will search for a device named MIDI\_DEV\_NAME, and will monitor key events from that device.

The application has been modified for a three octave spread keyboard

| C | D | E | F | G | A | B | C+1 |
|---|---|---|---|---|---|---|-----|
| q | w | e | r | t | y | u | i   |
| a | s | d | f | g | h | i |     |
| z | x | c | v | b | n | m |     |

| C# | Eb | F# | G# | Bb |
|----|----|----|----|----|
| 0  | -  | =  | ]  | \  |
| 5  | 6  | 7  | 8  | 9  |
| \`  | 1  | 2  | 3  | 4  |
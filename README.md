# fpick

**fpick** is an interactive file picker to traverse through directories tree in a terminal.

<div align="center">
    <a href="https://github.com/igrek51/fpick">GitHub</a>
    -
    <a href="https://crates.io/crates/fpick">Crates</a>
    -
    <a href="https://docs.rs/crate/fpick/">docs.rs</a>
</div>

## Installation
### Cargo
```sh
cargo install fpick
```
This will install `fpick` binary in Rust's Path.

### Binary
Alternatively, you can download the compiled binary:

```sh
curl -L https://github.com/igrek51/fpick/releases/download/0.2.0/fpick -o ~/bin/fpick
chmod +x ~/bin/fpick
```

## Usage
Launch the interactive file picker by running `fpick`.

Navigate with keyboard:

- Up & Down to move between files and directories,
- Left to go up,
- Right to enter a directory.
- Type in the phrase of a filename to filter the list of files
- Enter to select a file, exit and print its path to stdout.

You can use it in combination with other commands, for example to print the selected file:
```sh
cat `fpick`
```
or change directory interactively:
```sh
cd `fpick`
```

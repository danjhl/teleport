teleport is a commandline tool to quickly move to bookmarked directories.

# Usage

Mark the current directory as a bookmark with `tm -b bookmark`.
Now you can move there using `tp bookmark`.

You can also quickly mark directories with numbers using `tm` or `tm directory`.
After the message `Marked as 0` and you can move to `directory` using the index with `tp 0`.


# Installation

Build the binary with `cargo build` and make sure it is available in the terminal (e.g. add it to /usr/local/bin/teleport).

## Linux
Add the following functions to your .bashrc.

```
function tp() {
    cd $(teleport -g "$@")
}

function tm() {
    teleport "$@"
}
```

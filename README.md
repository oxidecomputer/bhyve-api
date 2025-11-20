# bhyve API

A Rust library interface to bhyve's kernel ioctl interface.

This library is unmaintained. In the subsequent years, illumos' bhyve
kernel interface has changed. This library has not been tested or
updated to match, and in both cases the interfaces to FreeBSD and
illumos `bhyve` have diverged.

The Rust bindings for illumos' bhyve as used in Propolis, the userland
VMM maintained by Oxide, are developed alongside Propolis and housed
in its repository.

## Prerequisites

The main requirement is Rust, the bhyve API library has been tested on
the version (1.42.0) packaged with OmniOS CE r151032. Install the
package `ooce/developer/rust`.

### bhyve

To run the tests or examples, you will also need bhyve installed. On
OmniOS CE these packages are `system/bhyve` and
`system/library/bhyve` (version 0.5.11-151032.0).

Once you've installed the bhyve packages, you can check if your
hardware is supported by running:

```
    pfexec bhhwcompat -v
```

## Examples

There are two example scripts included in `examples/`, one simple
command-line interface, and one demo that illustrates the features.
Both currently require root permissions, because they create real VM
devices. The demo takes no command-line arguments, and can be run as:

```
    sudo cargo run --example demo
```

The command-line example does take arguments, so can be run as:

```
    sudo cargo run --example tui -- create vmname
    sudo cargo run --example tui -- run vmname
    sudo cargo run --example tui -- destroy vmname
```

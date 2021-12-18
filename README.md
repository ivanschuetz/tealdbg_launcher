# tealdbg_launcher

![Build](https://github.com/ivanschuetz/tealdbg_launcher/actions/workflows/actions.yml/badge.svg)

Start the [TEAL debugger](https://developer.algorand.org/docs/features/asc1/debugging/#using-the-teal-debugger) from Rust.

Cargo.toml:
```
tealdbg = { git = "https://github.com/ivanschuetz/tealdbg_launcher" }
```

Insert this where you want to debug smart contract calls:

```rust
tealdbg::launch_default(
    &[my_tx1, my_tx2],
    "approval.teal",
)
```

To override defaults:
```rust
tealdbg::launch(
    Config {
        mode: tealdbg::Mode::Sandbox {
            command: "<path>/sandbox",
        }
        ..Config::default()
    },
    &[my_tx1, my_tx2],
    "approval.teal",
)
```

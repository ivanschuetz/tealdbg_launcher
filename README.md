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

To [override defaults](https://github.com/ivanschuetz/tealdbg_launcher/blob/12538a4522b8dcfb21484217429e09f503f0837b/src/lib.rs#L11):

```rust
tealdbg::launch(
    Config {
        node_dir: Some("<node dir>"),
        ..Config::default()
    },
    &[my_tx1, my_tx2],
    "approval.teal",
)
```

⚠️ Sandbox is not supported! If you need it, please open an issue.

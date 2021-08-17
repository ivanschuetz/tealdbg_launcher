# tealdbg_launcher

Start the [TEAL debugger](https://developer.algorand.org/docs/features/asc1/debugging/#using-the-teal-debugger) from Rust.

Insert this where you want to debug smart contract calls:

```rust
tealdbg_launcher::launch_default(
    &[my_tx1, my_tx2],
    "approval.teal",
)
```

To [override defaults](https://github.com/ivanschuetz/tealdbg_launcher/blob/12538a4522b8dcfb21484217429e09f503f0837b/src/lib.rs#L11):

```rust
tealdbg_launcher::launch(
    Config {
        node_dir: Some("<node dir>"),
        ..Config::default()
    },
    &[my_tx1, my_tx2],
    "approval.teal",
)
```

A River Windows Manager written in Rust.

Heavy thanks to the creator of [rrwm](https://github.com/cap153/rrwm), **cap153** and the creator of [canoe](https://github.com/roblillack/canoe), **robillack** whom projects really help me to understand the protocols and implementation of a WM. 
# Run

At this point, it doesn't work ^^, I haven't completed the Dispatcher and focused on bringing all the pieces of what I had in mind. So I have everything I need to begin with.

To run it with river (you'll get a dark screen but it might be needed for testing), run `cargo build` and then `river -c target/debug/kawa` and if you need the logs `RUST_LOG=<log level, usually info> river -c target/debug/kawa`

Or you can run it as is with `RUST_LOG=kawa=info cargo run --package kawam --bin kawa` but you're most likely to get block after creating the state of the WM.

The config file is read at `.config/kawa/kawa.lua` and there is no fallback yet so if you don't have at least an empty file, running it might (at a great certainty) not work.

A "default" config file to test a bit some of the already here lua bindings :

```lua
kw.print("Hello the config file has been executed")
kw.exec_cmd("$TERM")

kw.on("kawa.start", function ()
    kw.print("kawa has started")
    kw.print("Entering the main loop")
end)

kw.config({
    general = {
        yes = true,
    },
    animations = {
        enabled = true,
    },
})
```
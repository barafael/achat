# achat

[![Rust](https://github.com/barafael/achat/actions/workflows/rust.yml/badge.svg)](https://github.com/barafael/achat/actions/workflows/rust.yml)

A collection of simple modules which showcase simple use of tasks, channels, and other tokio primitives to implement simple networking applications.
Purely educational purposes.

## Documentation
Just run `cargo doc --open` :) there used to be a GH pages site with docs, which I linked to, apologize. It's gone now. I have rewritten the git history for this repo because it was just unwieldy large, sorry for the hassle :)

## echo
TCP clients connect to the server. The server returns each message they send back to them.

## collector
TCP clients connect to the server. The server collects each message they send in a central hashmap.
Each client can request this hashmap.

## chat
TCP clients connect to the server. The server collects each message they send and broadcasts it to all others.

## chat_with_announce
TCP clients connect to the server. The server collects each message they send and broadcasts it to all others.
In a regular interval, the server announces the uptime to everyone.

## chat_with_cancel
TCP clients connect to the server. The server collects each message they send and broadcasts it to all others.
The entire application can be terminated with a specific line written to the client.

## dump_client
TCP clients connect to the server. Whatever they send, the server will dump on its `stdout`.

## dump_client_gui
TCP clients connect to the server. Whatever they send, the server will dump on its `stdout` (same as `dump_client`, except this one has a GUI).

## client
Opens TCP connection to a server, connects that stream to stdin and stdout.

## There are more, just have a look

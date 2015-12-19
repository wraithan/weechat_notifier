# weechat-notifier

Get a local notification from your remote [weechat](https://weechat.org) session! 

## Goals

My goal with this project is to build a rust version of
[mythmon/page](https://github.com/mythmon/page). Mostly because I wanted to
write a parser for weechat's
[relay protocol](http://weechat.org/files/doc/stable/weechat_relay_protocol.en.html)
but a little because if you use a command like LIST on irc.freenode.org it would
heat up one of your cores for a while and you would stop receiving notifications
for a couple minutes.

## Modules

Currently there are 3 modules in this repo. `weechat-notifier`,
`weechat-client`, and `weechat-parser`. If any of these gain more adoption
outside of my use cases I'll break them out into their own repo to enable more
clear communication and development history.

### `weechat-notifier`

This is the root module. It will provide the command line interface to starting
the daemon as well as the daemon itself. This is where any further modules will
likely grow out of.

### `weechat-client`

This module handles getting a connection to the weechat relay server, hooking it
up to the parser and maintaining any state needed to keep the connection alive
as well as reconnect if desired. This is what most people will use instead of
directly using the `weechat-parser` module.

Currently it just connects emits events on a
[MPSC](https://doc.rust-lang.org/std/sync/mpsc/) channel.

It will grow to have more capabilities to send commands to the server and
filtering around emitted events.

### `weechat-parser`

Split into its own module for conceptual/develmental reasons this module is also
more useful to folks who don't like the connection or threading modules used by
`weechat-client` and want to build their own.

This module currently works via MPSC channels as well. It takes 2, a receiving
end and a transmitting end. This way it can interact with pretty much any
network library and many types of abstractions are possible on top of the
emitted event stream.

# Loco
### **Loc**al d**o**

This is a hacky, opinionated project to recreate a python tool I have in Rust. Loco will eventually wrap a whole bunch of commands for moving files from, and opening files on, remote systems. At the moment, it just wraps `rsync` to quickly copy files and directories to your local machine.

---

**WARNING**: This is very much a work in progress project for learning some Rust. It should not be relied upon in any manner and may cause data loss or corruption. Use with care and always backup your data (host and remote) first.

---

## Installation

Ensure you have the rust toolchain installed. Then:

```
git clone https://github.com/smutch/loco.git
cd loco
cargo install --path .
```

## Setup

Ensure that you ssh into your remote machine with ssh-agent forwarding enabled (`-A` flag) and remote port forwarding to `localhost:22` (e.g. `-R 8787:localhost:22`). It's typically easiest to set these options appropriately in your host's `.ssh/config` file. e.g.

```
Host skippy
HostName skippy.is.here.com
RemoteForward 8787 localhost:22
ForwardAgent yes
```

On your remote machine, you will need to create a config file for Loco at `$XDG_CONFIG/loco/config.toml` (typically `~/.config/loco/config.toml`) with some defaults:

```
port = 8787
username = "YOUR USERNAME"
dest = "~/Downloads"
```

In the future, Loco will create this for you if you haven't already.

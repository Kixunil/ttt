# Trivial time tracker

Free and Open Source minimalistic time tracker leveraging QubesOS VMs to categorize the tasks.

## About

This tool is written for QubesOS, which separates all your activities into separate virtual machines (domains) in order to achieve security. As an interesting byproduct, it achieves nice organization of life as well. If the separation mechanism is used properly, there exists mapping from `vm_name -> fun|work` (or a more granular categorization). If the mapping doesn't seem to work for you, you've almost certainly mixed security domains that you shouldn't have mixed. :)

There are two notable edge cases: `dom0`, which can be considered neutral and `dispX` which has unpredictable name. DispVMs are currently prefixed by the name of their parents followed by colon to help with categorization, however, it's not that useful for `dom0`. Contributions to improve it welcome! (Ideas: ask about it or decide based on the template.)

## Installation

0. The tools are written in Rust. `sudo apt install cargo` or `sudo dnf install` should be sufficient. Build it by separately entering the directories `cli`, `qubes_rpc` and running `cargo build --release`. Binaries are in `target/release/`
1. Copy the RPC service binary into `/usr/local/etc/qubes-rpc/ttt` and cli tool into `/usr/local/bin/ttt` of the VM that will handle the incoming data. It should be a VM that you launch often, of course.
2. Securely copy the time track script into `dom0` and review that it doesn't do anything malicious. Configure the VM you want to use, then set it up to run automatically. (Menu -> System tools -> Session and Startup -> Application Autostart)
3. Launch the tracking script in dom0 or reboot the computer

## Usage

### Stats

After collecting some data you can run `ttt stats today` to see todays statistics. Other options include:

* `ttt stats yesterday`
* `ttt stats this week`
* `ttt stats this month`
* `ttt stats this year`
* `ttt stats BEGIN`
* `ttt stats BEGIN END`

Where `BEGIN` and `END` is any string that can be parsed as local date/time by [`chrono`](https://crates.io/crates/chrono)

### Groups

You can group VMs in order to simplify the output of stats:

1. Create a group by running `ttt group create GROUPNAME` (e.g. `ttt group create fun`)
2. Add VMs to the group by running `ttt group addvms GROUPNAME VMS ...` (e.g. `ttt group addvms fun games social-networks`)
3. After running `ttt stats ...` you'll see the VMs grouped. They currently can't be "ungrouped" ping me or make a PR, if you need it.

## Troubleshooting

In case something seems wrong, inspect the log file in `~/.local/share/ttt/qubes_rpc.log`

## Contributions and planned features

I'll be happy to accept contributions, especially to the CLI tool!

### Wanted features

* Filter VMs
* ~~Groups of VMs (sometimes it's useful to separate an activity into several qubes - e.g. work between each client/project)~~
* Handle edge-cases like going to sleep, turning off the computer, turning off tracking VM
* Have more information about dispvm launched by `dom0`
* Allow merging output of `ttt stats` for dispvms
* Combine with other sources of events (smartphone, manual entry)
* Allow editing/overriding the records
* Proper documentation (man page etc)
* Shell completion
* Pre-compute statistics, if needed (not sure how it'll behave after recording many events)
* Cleanup and documentation of the Rust code

## Known issues

~~UNIQUE constraints are violated sometimes, IDK how's that possible. Some events/VMs seem to be lost.~~ - Looks like I messed up replacing the binaries after fixing the issue keeping it here until better tested.

## License

MITNFA

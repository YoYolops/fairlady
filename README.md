Building blocks for a replicant network. Its main goal i storing data from a client into a server. Focused on safety (end to end encrypting) and scalability. 

# Similar projects:
[Nextcloud Files](https://google.com): Solid and mature. Written in `php`. Centralized. Focused on the end user (companies)
[Syncthing](https://syncthing.net/): `C++`, Descentralized, secure

# Marks
## MARK I: Oct 19, 2025;
Today we finished our first baby step!

We still don't have enough to call it a PoC, but i'm pretty sure we are halfway through.
These first couple hundred lines were very important to understand a little better what we are doing and what will be, most likely, the future challenges. As of right now, everything seems technically possible and overall manageable within the remaining estimated time (12 months).

Lets talk about the **MARK I** features and upgrades:
1. We successfully implemented a file watcher, with the notify crate. We watch a given folder for every change, by using a syscall, OS dependant and managed by notify

2. We created the foundation for NimbusProtocol, our inter application (client/server) protocol, with simplicity in mind (but not enough to use some already existent ones).

I think that's worth a release, consider ourselves in client@0.1.1

# DECISIONS:
1. Using big endian to serialize `NimbusProtocol`

### How to detect real changes?
Thats hard...
Firstly, we are goig to encrypt, somehow, the user data. As of right now, we have two main strategies:
    1. Any change in the watched folder fires a server request with the entire folder content ecrypted.
        - Performance issues, specially for huge file trees
    2. We choose to only encript data itself, preserving folder structure and file/folder names, and detecting changes by traversing the file tree on events.

# To do
1. **ACCOUNT FOR SILENT REMOVAL:** When a file/folder is sent to the trash bin, it fires a `Modify(Name(From))`, instead of a Remove event. So it won't be catched by the Remove match arm. In addition, the app does not send a request to server on `Modify(Name(From))`, only on `Modify(Name(Both))`, which represents a rename. In other words, sending a file to the trash bin would be completely missed by our system. This shows the need for an extra algorithm to ensure consistency between client and server

2. [SOLVED] - **FIX PATHSTRING ARRIVING BADLY FORMATTED ON THE SERVER:** The paths strings for some request kids are arriving badly formatted on the server

3. **TEST IN WINDOWS:** We are, in a first moment, building everything entirely for **LINUX**. No tests were made in another O.S., and it is very important to do so, since different file system might result in remarkable differences in notifications fired.

4. **FIX BROKEN PIPE:** when connection with the server is lost, client fails almost silently. TCP connection loss is not handled. The following error will be printed by the client in such event: `Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }`

5. **IMPROVE MAIN FUNCTION:** currently, it does not account for it's main task's health. Main tasks are core to the system. They must be watched, if any task returns with an error, the entire app might need to stop and log the error. Otherwise, main tasks may fail silently.

6. **INFORM THE AMOUNT OF BYTES TRANSITING VIA TCP CONNECTION:** otherwise the other end won't know how many bytes should decode and might try to parse an invalid sequence

7. **Send real data in tcp connection:** Curretly, NimbusProtocol has, in multiple of its variants, a data field that is currently mocked. It must dinamically identify the target file and collect the relevant data to be sent


### Commit Classes:
Prototype:
Feature:
Improve:
Update:
Fix:
Release:

# FONTS:
large scale folder watching: https://github.com/notify-rs/notify/issues/412
Linux Docs: https://man7.org/linux/man-pages/man7/inotify.7.html
Linux Docs 2(?): https://docs.kernel.org/filesystems/inotify.html
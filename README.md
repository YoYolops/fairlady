Uma rede replicante. Seu objetivo principal é armazenar dados de um client em um server, com segurança (criptografia de ponta a ponta) e escalabilidade;

### FONTS:
large scale folder watching: https://github.com/notify-rs/notify/issues/412

# DECISIONS:
1. Using big endian to serialize `NimbusProtocol`

### How to detect real changes?
Thats hard...
Firstly, we are goig to encrypt, somehow, the user data. As of right now, we have two main strategies:
    1. Any change in the watched folder fires a server request with the entire folder content ecrypted.
        - Performance issues, specially for huge file trees
    2. We choose to only encript data itself, preserving folder structure and file/folder names, and detecting changes by traversing the file tree on events.

### Known Issues
1. When a file/folder is sent to the trash bin, it fires a `Modify(Name(From))`, instead of a Remove event. So it won't be catched by the Remove match arm. In addition, the app does not send a request to server on `Modify(Name(From))`, only on `Modify(Name(Both))`, which represents a rename. In other words, sending a file to the trash bin would be completely missed by our system. This shows the need for an extra algorithm to ensure consistency between client and server

2. The paths strings for some request kids are arriving badly formatted on the server
2. SOLUTION: We were sending the struct across the network. We need to parse it before sending
### Commit Classes:
Prototype:
Feature:
Improve:
Update:
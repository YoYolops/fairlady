Uma rede replicante. Seu objetivo principal é armazenar dados de um client em um server, com segurança (criptografia de ponta a ponta) e escalabilidade;

### FONTS:
large scale folder watching: https://github.com/notify-rs/notify/issues/412

# DECISIONS:

### How to detect real changes?
Thats hard...
Firstly, we are goig to encrypt, somehow, the user data. As of right now, we have two main strategies:
    1. Any change in the watched folder fires a server request with the entire folder content ecrypted.
        - Performance issues, specially for huge file trees
    2. We choose to only encript data itself, preserving folder structure and file/folder names, and detecting changes by traversing the file tree on events.

### Commit Classes:
Prototype:
Feature:
Improve:
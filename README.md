![Fairlady logo. The birth of venus](./docs/fairlady.png)

# Fairlady

An IPFS plugin that allows safe data storage over the network, by providing tooling for data encryption/decryption and storage management.



### TO DO:
(Currently, at every run, new aes key is created)
1. ✅ DONE: Create a function to check if credentials are already created inside ./data;
2. ✅ DONE: Create a function to store created credentials inside ./data
3. 🟡 Build strategy to dinamically get the latest data
4. 🟡 Create folder watcher


### To be known:
Keys are stored with compactly, which approaches data encoding/decoding in such a way that will always
be able to parse whats inside the keys file, even emptyness.


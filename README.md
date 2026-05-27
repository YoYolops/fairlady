![Fairlady logo. The birth of venus](./docs/fairlady.png)
***Fairlady is a prototype.** 

# Fairlady

An IPFS plugin that allows safe data storage over the network, by providing tooling for data encryption/decryption and basic storage management.

The goal is exploring IPFS capabilities and limitations via its open source tooling, while 
adressing privacy concerns when using the network as a global CDN.

Fairlady allows one to use the Inter-Planetary File System to safely store data. Once stored,
with the IPFS node correctly set, online, and with `.fairlady` credentials in hand, user can access 
it from anywhere in the world. No need to enroll in any third party paid services: you store it, 
you own it.

### TO DO:
1. ✅ DONE: Create a function to check if credentials are already created inside ./data;
2. ✅ DONE: Create a function to store created credentials inside ./data
3. 🔴 FROZEN DUE TO IRRELEVANCE FOR NOW: Build strategy to dinamically get the latest data
4. ✅ Create folder watcher
5. ✅ build CLI
6. ✅ Extend glifo with chacha20-poly1305
7. ✅ Extend glifo with two-fish
8. ✅ Extend glifo with serpent

🟡 pending
✅ done
🟣 frozen
🔴 a (very) hard problem


### To be known:

Keys are stored with compactly, which approaches data encoding/decoding in such a way that will 
always be able to parse whats inside the keys file, even emptyness.


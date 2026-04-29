### TO DO:
(Currently, at every run, new aes key is created)
1. DONE: Create a function to check if credentials are already created inside ./data;
2. DONE: Create a function to store created credentials inside ./data
3. How another node will fetch the same data

### To be known:
Keys are stored with compactly, which approaches data encoding/decoding in such a way that will always
be able to parse whats inside the keys file, even emptyness.
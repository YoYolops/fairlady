# Proposal:
Cumulus as a communitary backup system, to be built on top of [iroh](https://github.com/n0-computer/iroh), an [IPFS implementation](https://docs.ipfs.tech/concepts/ipfs-implementations/#popular-node-implementations-and-tools).

Imagine you are an archiver of some sort and need to keep a backup of your files. You buy a couple of hard-drives and manage to copy your entire collection to them. But, beware, this data is very precious to you and, to increase safety, you think it's a good a idea to have this copy in another part of the world.

Your alternatives are:
1. Pay some SaaS to do it;
2. Leverage cloud infrastructure to store your data remotely;
3. Physically move the storage hardware somewhere else;
4. With multiple computers in hand, configure an open source software to sync/store data ([Syncthing](https://syncthing.net/), [Copyparty](https://github.com/9001/copyparty))

Cumulus presents itself as an alternative. You set up your node and cumulus will do two things:

1. Ensure you will give what you take: This means that if you need to use 1GB of network's space, you have to give 1GB of free space to be used by others;

2. Manage relationship with peers;

# Data flow:

A client node suddenly comes to life, lets call him Adam. It starts by checking a special broker node, that stores other nodes available spaces.

Adam asks for the broker to publish it's available space and then gathers information about which other nodes it will have to call in order to store its data. If Adam wants to store 1GB worth of data, it might find a peer with enough space to store everything, or it might use two peers, with 500MB each.

Once adam found the necessary peers, it connects to them directly, via iroh connections. After stablishing communication, Adam sends a intial request informing how many of the peer storage it will need. The peer checks if it is really able to store that amount of data and responds, allowing the transfer or not.

If transfer is not allowed, Adam calls broker again to try another node. 

If transfer is allowed, Adam sends its data to peer.

There are many complexities that arise from this. The flow presented is just a sketch and is not meant to be perfect. It is just a prototype.

We can map and address these problems one by one, as time allows, until we feel confortable with the result we get.


FROM ipfs/kubo:latest

# Set the IPFS profile to 'server'. 
# Prevents node from broadcasting local discovery pings, 
ENV IPFS_PROFILE=server

# Expose the necessary ports
# 4001: Swarm (TCP/UDP for P2P network traffic)
# 5001: API (RPC commands)
# 8080: Gateway (HTTP access to content)
EXPOSE 4001/tcp 4001/udp 5001/tcp 8080/tcp

# Copy custom initialization scripts
# The official Kubo image automatically executes any .sh scripts placed in /container-init.d/
COPY init-config.sh /container-init.d/001-init-config.sh

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ipfs id || exit 1
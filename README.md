# achat
Super simple examples of network applications using tokio. Purely educational purposes.

# echo
TCP clients connect to the server. The server returns each message they send back to them.

# collector
TCP clients connect to the server. The server collects each message they send in a central hashmap.
Each client can request this hashmap.

# chat
TCP clients connect to the server. The server collects each message they send and broadcasts it to all others.

# chat_with_announce
TCP clients connect to the server. The server collects each message they send and broadcasts it to all others.
In a regular interval, the server announces the uptime to everyone.

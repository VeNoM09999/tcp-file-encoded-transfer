# Overview

This project is to solidify my worldview on dealing with chunked file. In this project I have implement compressed chunks  transfer from user -> server.

# Client
Client sends chunks of compressed file (Gzip) over TCP protocol.

# Server (tungstenite | Non-Async)
Server receives and handle differnt matchs like Received Text message or Ping or Binary Data. Depending on it the server
creates file writer or finish(flush) the pending writes.  

This spawns a thread for each connection to handle file transfer. This is not recommed as many users joining at the same time can easily use all the avaialble resource. But this isn't made for performace in mind but for learning working with binary data. 

# Frontend (SolidJs)
Will be uploading Frontend code as well soon.

Its a very basic ui that supports drag and drop support for files and its made in SolidJS. 
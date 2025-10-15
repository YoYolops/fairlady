Similar to what i did in (my previous project), i am going to keep a diary with the baby steps needed to finally build this project. I have been thinking about it for quite a while now, and i've finally agreed to myself about the initial scope. 
The idea is building a tool that will serve as a data replicant.:

*Client* watches an OS folder for changes. Everytime it does, we send the updates to a *Server* and keep everything in sync.
Whe aim to assure security (no uncrypted data leaves client machine), and scalability (hopefullly rust + tokio will bring that from the get go).
# league_client_connector

Rust implementation for [lcu-connector](https://github.com/Pupix/lcu-connector) minus the
file watching mechanism. This crate needs the League Client to be opened, in order to get the
installation path for League of Legends so the `lockfile` can be retrieved correctly.

Note that every time the League Client is opened, it creates a new `lockfile` so a watcher or
some refresh mechanism needs to be implemented to use correctly in an application.

The contents of the `lockfile` are parsed and presented in a readable format so a connection to
the [Game Client API](https://developer.riotgames.com/docs/lol#game-client-api) can be
established.

## Roadmap

- [x] Read lockfile
- [x] Error handling
- [x] Documentation
- [ ] File watcher?

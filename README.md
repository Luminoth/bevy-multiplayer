# Bevy Multiplayer

Playing around with a mix of Bevy, Agones, and hopefully OpenMatch

# Notes

* Run game server on [Agones](https://agones.dev/site/docs/overview/)
  * [Example game server](https://github.com/googleforgames/agones/tree/release-1.44.0/examples/simple-game-server)
  * [State Diagram](https://agones.dev/site/docs/reference/gameserver/#gameserver-state-diagram)
* Matchmaking with [OpenMatch](https://open-match.dev/site/docs/overview/)
  * For now use [tonic](https://github.com/hyperium/tonic) for gRPC, Google is working on a native impl based on this
* [Rapier](https://rapier.rs/) for physics
  * [Character controller](https://rapier.rs/docs/user_guides/bevy_plugin/character_controller/)
* [bevy_replicon](https://crates.io/crates/bevy_replicon) and [bevy_replicon_renet](https://crates.io/crates/bevy_replicon_renet) for multiplayer networking
* Potentially look at [SpaceEditor](https://crates.io/crates/space_editor) again for an editor
* [bevy-ui-navigation](https://crates.io/crates/bevy-ui-navigation/) for UI navigation
  * TODO:
* TODO: when bevy 0.15 is out, make use of TerminalCtrlCHandlerPlugin to gracefully shutdown on ctrl+c

## Running Locally

* Put the Agones SDK Server binary in /bin
* From the workspace root run `bin/agones-local.sh`

* TODO: replace Rapier with [Avian](https://crates.io/crates/avian3d)
  * [Tnua Character Controller](https://github.com/idanarye/bevy-tnua)

* TODO: organize libs
  * client_shared -> client + services shared (messages mainly)
  * server_shared -> server + services shared (messages mainly)
  * internal -> services shared (messages, shared utils, etc)
  * game -> client + server shared (messages, gameplay, shared utils, etc)
  * can have a common lib if everything needs shared utils and such

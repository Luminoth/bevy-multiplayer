# Bevy Multiplayer

Playing around with a mix of Bevy, Agones, and hopefully OpenMatch

# Notes

* Run game server on [Agones](https://agones.dev/site/docs/overview/)
* Matchmaking with [OpenMatch](https://open-match.dev/site/docs/overview/)
  * For now use [tonic](https://github.com/hyperium/tonic) for gRPC, Google is working on a native impl based on this
* [Rapier](https://rapier.rs/) for physics
  * [Character controller](https://rapier.rs/docs/user_guides/bevy_plugin/character_controller/)
* [bevy_replicon](https://crates.io/crates/bevy_replicon) and [bevy_replicon_renet](https://crates.io/crates/bevy_replicon_renet) for multiplayer networking
* Potentially look at [SpaceEditor](https://crates.io/crates/space_editor) again for an editor
* [bevy-ui-navigation](https://crates.io/crates/bevy-ui-navigation/) for UI navigation
  * TODO:

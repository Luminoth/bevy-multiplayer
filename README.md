# Bevy Multiplayer

Playing around with a mix of Bevy, Agones, and hopefully OpenMatch

# Notes

* Run game server on [Agones](https://agones.dev/site/docs/overview/)
* Matchmaking with [OpenMatch](https://open-match.dev/site/docs/overview/)
  * For now use [tonic](https://github.com/hyperium/tonic) for gRPC, Google is working on a native impl based on this
* [Rapier](https://rapier.rs/) for physics
  * [Character controller](https://rapier.rs/docs/user_guides/bevy_plugin/character_controller/)
* [Naia](https://github.com/naia-lib/naia) for multiplayer networking
  * Depending on if I can even get this to build or not
* Potentially look at [SpaceEditor](https://crates.io/crates/space_editor) again for an editor

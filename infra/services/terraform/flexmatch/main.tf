# TODO: support multiple queues (fallbacks, etc)
# attach all to the one configuration

resource "awscc_gamelift_matchmaking_rule_set" "rule_set" {
  name = "${var.name}-rule-set"

  rule_set_body = var.rule_set
}

resource "awscc_gamelift_matchmaking_configuration" "matchmaking_configuration" {
  name = "${var.name}-matchmaking-configuration"

  acceptance_required     = var.acceptance_required
  request_timeout_seconds = var.timeout

  rule_set_name = awscc_gamelift_matchmaking_rule_set.rule_set.id

  flex_match_mode = "WITH_QUEUE"

  game_session_queue_arns = [
    var.queue_arn,
  ]
}

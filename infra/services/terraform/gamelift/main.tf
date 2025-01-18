# TODO: support multiple fleets (different regions, etc)
# attach all to the one queue

data "aws_iam_policy_document" "fleet_assume_role" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["gamelift.amazonaws.com"]
    }
  }
}

resource "awscc_iam_role" "fleet_role" {
  role_name                   = "gamelift-fleet-role"
  assume_role_policy_document = data.aws_iam_policy_document.fleet_assume_role.json
  description                 = "IAM role for GameLift Fleet"
  policies = [{
    policy_name = "fleet-policy"
    policy_document = jsonencode({
      Version = "2012-10-17"
      Statement = [
        {
          Effect = "Allow"
          Action = [
            "s3:GetObject",
            "s3:ListBucket",
            "logs:CreateLogGroup",
            "logs:CreateLogStream",
            "logs:PutLogEvents",
            "logs:DescribeLogGroups",
            "logs:DescribeLogStreams"
          ]
          Resource = "*"
        }
      ]
    })
  }]

  tags = {
    Name = "terraform-${var.name}"
  }
}

resource "awscc_gamelift_container_group_definition" "container_group" {
  name = "${var.name}-fleet"

  operating_system = var.operating_system

  total_cpu_limit    = var.cpu_limit
  total_memory_limit = var.memory_limit

  container_definitions = [{
    container_name      = var.name
    container.image_uri = var.image_uri

    port_configuration = {
      container_port_ranges = var.port_ranges
    }
  }]

  tags = {
    Name = "terraform-${var.name}"
  }
}

resource "awscc_gamelift_container_fleet" "fleet" {
  fleet_role_arn = awscc_iam_role.fleet_role.arn
  description    = "GameLift Fleet for ${var.name}"

  instance_type = var.instance_type
  billing_type  = var.billing_type

  tags = {
    Name = "terraform-${var.name}"
  }
}

resource "awscc_gamelift_game_session_queue" "queue" {
  name = "${var.name}-queue"

  destinations = [
    {
      destination_arn = awscc_gamelift_container_fleet.fleet.fleet_arn
    }
  ]

  timeout_in_seconds = var.queue_timeout

  tags = {
    Name = "terraform-${var.name}"
  }
}

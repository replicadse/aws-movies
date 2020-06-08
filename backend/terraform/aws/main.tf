resource "aws_iam_role" "lambda" {
  name = "aws-movies--lambda-role"
  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "lambda.amazonaws.com"
      }
    }
  ]
}
EOF
}

resource "aws_iam_policy" "aws-movies--lambda-logging" {
  name        = "aws-movies--lambda-logging"
  path        = "/"
  description = "IAM policy for logging from a lambda"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*",
      "Effect": "Allow"
    }
  ]
}
EOF
}

resource "aws_cloudwatch_log_group" "aws-movies--movies-handler-graphql-log" {
  name              = "/aws/lambda/${aws_lambda_function.aws-movies--movies-handler-graphql.function_name}"
  retention_in_days = 7
}

resource "aws_iam_role_policy_attachment" "lambda-logs" {
  role       = aws_iam_role.lambda.name
  policy_arn = aws_iam_policy.aws-movies--lambda-logging.arn
}

resource "aws_lambda_function" "aws-movies--movies-handler-graphql" {
  filename      = "../../functions/movies-handler-graphql/target/bootstrap.zip"
  function_name = "aws-movies--movies-handler-graphql"
  role          = aws_iam_role.lambda.arn
  handler       = "bootstrap"
  source_code_hash = filebase64sha256("../../functions/movies-handler-graphql/target/bootstrap.zip")
  runtime = "provided"

  environment {
    variables = {
      TABLE_NAME = "aws-movies--movies"
    }
  }
}

resource "aws_dynamodb_table" "movies-table" {
  name           = "aws-movies--movies"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "pk"
  range_key      = "sk"

  point_in_time_recovery {
    enabled = true
  }

  attribute {
    name = "pk"
    type = "S"
  }
  attribute {
    name = "sk"
    type = "S"
  }
  // attribute {
  //   name = "published_at"
  //   type = "S"
  // }
  // attribute {
  //   name = "published_year"
  //   type = "N"
  // }
  // attribute {
  //   name = "title"
  //   type = "S"
  // }
  // attribute {
  //   name = "imdb_id"
  //   type = "S"
  // }

  global_secondary_index {
    name               = "GSI-1"
    hash_key           = "sk"
    range_key          = "pk"
    projection_type    = "INCLUDE"
    non_key_attributes = ["pk", "sk", "characters"]
  }
  // global_secondary_index {
  //   name               = "GSI-2"
  //   hash_key           = "title"
  //   range_key          = "published_at"
  //   projection_type    = "INCLUDE"
  //   non_key_attributes = ["pk", "sk", "imdb_id"]
  // }
  // global_secondary_index {
  //   name               = "GSI-3"
  //   hash_key           = "imdb_id"
  //   projection_type    = "INCLUDE"
  //   non_key_attributes = ["pk", "sk", "imdb_id", "published_at", "title"]
  // }
  // global_secondary_index {
  //   name               = "GSI-4"
  //   hash_key           = "published_year"
  //   projection_type    = "INCLUDE"
  //   non_key_attributes = ["pk", "sk", "published_year", "published_at", "title"]
  // }
}

resource "aws_iam_policy" "dynamodb" {
  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Action": [
          "dynamodb:Query",
          "dynamodb:PutItem",
          "dynamodb:GetItem"
      ],
      "Resource": "*"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "dynamodb" {
  role       = aws_iam_role.lambda.name
  policy_arn = aws_iam_policy.dynamodb.arn
}

resource "aws_iam_role" "movies-handler-graphql-iam" {
  name = "movies-handler-graphql-iam"
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

resource "aws_lambda_function" "movies-handler-graphql" {
  filename      = "../../functions/movies-handler-graphql/target/bootstrap.zip"
  function_name = "movies-handler-grapqhl"
  role          = aws_iam_role.movies-handler-graphql-iam.arn
  handler       = "bootstrap"
  source_code_hash = filebase64sha256("../../functions/movies-handler-graphql/target/bootstrap.zip")
  runtime = "provided"

  environment {
    variables = {
      TABLE_NAME = "movies"
    }
  }
}

resource "aws_dynamodb_table" "movies-table" {
  name           = "movies"
  billing_mode   = "PAY_PER_REQUEST"
  read_capacity  = 1
  write_capacity = 1
  hash_key       = "id"

  attribute {
    name = "id"
    type = "S"
  }
  attribute {
    name = "title"
    type = "S"
  }

  global_secondary_index {
    name               = "title"
    hash_key           = "title"
    write_capacity     = 1
    read_capacity      = 1
    projection_type    = "KEYS_ONLY"
  }
}

resource "aws_iam_policy" "dynamodb" {
  name   = "api-movies-dynamodb"
  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Action": [
          "dynamodb:Scan",
          "dynamodb:PutItem",
          "dynamodb:GetItem",
          "dynamodb:DeleteItem"
      ],
      "Resource": "*"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "dynamodb" {
  role       = aws_iam_role.movies-handler-graphql-iam.name
  policy_arn = aws_iam_policy.dynamodb.arn
}

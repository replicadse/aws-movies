provider "aws" {
  version = "~> 2.8"
  region = "eu-central-1"
}

terraform {
  backend "s3" {
    bucket = "776479404968-terraform"
    #dynamodb_table = "776479404968-terraform-lock"
    encrypt = true
  }
}

variable "project" {
  type = string
}

variable "stage" {
  type = string
}

resource "aws_iam_role" "iam_for_lambda" {
  name = "iam_for_lambda"
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
  filename      = "../../target/movies-handler-graphql.zip"
  function_name = "${var.project}--${var.stage}--movies-handler-grapqhl"
  role          = aws_iam_role.iam_for_lambda.arn
  handler       = "bootstrap"
  source_code_hash = filebase64sha256("../../target/movies-handler-graphql.zip")
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
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.dynamodb.arn
}

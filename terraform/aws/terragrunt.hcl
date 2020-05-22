remote_state {
  backend = "s3"
  
  generate = {
    path      = "backend.generated.tf"
    if_exists = "overwrite_terragrunt"
  }

  config = {
    bucket         = "${get_env("TF_VAR_account")}--${get_env("TF_VAR_project")}--${get_env("TF_VAR_region")}--${get_env("TF_VAR_stage")}--terraform"
    key            = "${path_relative_to_include()}/terraform.tfstate"
    region         = "${get_env("TF_VAR_region")}"
    encrypt        = true
    dynamodb_table = "${get_env("TF_VAR_account")}--${get_env("TF_VAR_project")}--${get_env("TF_VAR_region")}--${get_env("TF_VAR_stage")}--terraform-lock"
  }
}

generate "provider" {
  path = "provider.generated.tf"
  if_exists = "overwrite_terragrunt"
  contents = <<EOF
provider "aws" {
  version = "~> 2.8"
  region = "${get_env("TF_VAR_region")}"
}
EOF
}

# aws-movies

## Environment variables

|Variable|Description|Mandatory|Used in Makefile command|
|--- |--- |--- |--- |
| AWS_ACCESS_KEY_ID | AWS Athentication and Authorization | yes | [`deploy`, `destroy`, `test-aws`] |
| AWS_SECRET_ACCESS_KEY | AWS Athentication and Authorization | yes | [`deploy`, `destroy`, `test-aws`] |
| TF_VAR_region | AWS region to deploy to | yes | [`deploy`, `destroy`] |
| TF_VAR_account | AWS Account ID | yes | [`deploy`, `destroy`] |
| TF_VAR_project | Project name | yes | [`deploy`, `destroy`] |
| TF_VAR_stage | Stage of the deployment | yes | [`deploy`, `destroy`] |

## Recommended usage

For development, create a shell script that sets your env variables (like in `./terraform/aws/env_vars_sample.sh`). The name `./terraform/aws/env_vars.sh` is already in the `.gitignore`.\
For the AWS credentials, use [aws-vault](https://github.com/99designs/aws-vault).\
Sample command for terraforming could look like:
```
source ./terraform/aws/env_vars.sh && aws-vault exec ${USER_NAME} -- make deploy
```
Testing can then be done with:
```
aws-vault exec ${USER_NAME} -- make test
```

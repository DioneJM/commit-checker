# Development
## Prerequisites
- An `SNS_TOPIC_ARN` [environment variable](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html) must be set for the SNS topic that you want to use
- AWS Lambda must have permission to publish on SNS ARN
## Create Stack
```bash
sam deploy -t package.yaml --guided
```
note: requires samconfig.toml - [see docs for more info](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html)

## How to build and release:
Run the release.sh script under scripts/ directory
From project root:
```bash
./scripts/release.sh
```

## How to invoke lambda
```bash
aws lambda invoke --cli-binary-format raw-in-base64-out --function-name commit-checker-stack-CommitChecker-lVlrMRBDMr6X --payload '{"date": "2022-01-02T18:44:49Z" }' output.json && cat output.json
```

## Clean up
### Delete Stack
```bash
aws cloudformation delete-stack --stack_name <value>
```
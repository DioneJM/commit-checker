# Development
## Create
```bash
sam deploy -t package.yaml --guided
```
note: requires samconfig.toml - [see docs for more info](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-config.html)

## How to build and release:
1. Build new binary
```bash
cargo zigbuild --release --target aarch64-unknown-linux-gnu 
```
2. Copy binary into build/bootstrap
```bash
cp ./target/aarch64-unknown-linux-gnu/release/commit-checker ./build/bootstrap  
```
3. Deploy to AWS
```bash
sam deploy
```

## How to invoke lambda
```bash
aws lambda invoke --cli-binary-format raw-in-base64-out --function-name commit-checker-stack-CommitChecker-lVlrMRBDMr6X --payload '{"firstName": "Dione" }' output.json && cat output.json
```

## Clean up
### Delete Stack
```bash
aws cloudformation delete-stack --stack_name <value>
```
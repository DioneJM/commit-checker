# Expected date format is YYYY-MM-DD

date=$1
payload="{\"date\": \""${1}"T00:00:00Z\" }"
echo $payload
echo $date

if [[ "$date" =~ [0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    aws lambda invoke --cli-binary-format raw-in-base64-out \
        --function-name commit-checker-stack-CommitChecker-lVlrMRBDMr6X \
        --payload "$payload" \
        output.json && cat output.json
else
    echo "No valid date argument found. Expected format is YYYY-MM-DD"
fi

# Motorshop

## Description

Creation of an AWS Serverless application.

## WARNING !!!

The implementation could generate costs on AWS. The project is only for personal documentation and demonstration
purpose.

## Prerequisite

* NodeJS / NPM
* AWS Account / AWS CLI
* https://www.cargo-lambda.info/guide/getting-started.html

## Creation of AWS Infrastructure.

The application is defined as IAC based on AWS CDK.

Creation:

```
cdk init app --language=typescript
```

## Publish Cloud Foundation Package

```
cdk deploy 
```

![Alt text](./images/overview.png?raw=true "Cloud Formation")

## Test

```
POST https://???????????.execute-api.eu-west-1.amazonaws.com/prod/
Content-Type: application/json

{
  "name": "Peppo",
  "model": "V1 Turbo"
}
```

The following points could be investigated:

* CloudWatch (Logs)
* SNS / SQS / Lambda (Logs)
* S3 (Bucket containing a PDF)
* DynamoDB containing a Data Entry

![Alt text](./images/publish_model.png?raw=true "Publish Model")

![Alt text](./images/create_prospect.png?raw=true "Create Prospect")

## Remove / Destroy Artifacts

```
cdk destroy
```

Validate that all resources are removed from AWS.
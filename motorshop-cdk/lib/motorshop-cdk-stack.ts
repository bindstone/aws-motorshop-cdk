import * as cdk from 'aws-cdk-lib';
import {RemovalPolicy} from 'aws-cdk-lib';
import {Construct} from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3'
import {RustFunction} from "rust.aws-cdk-lambda";
import {EndpointType, LambdaRestApi} from "aws-cdk-lib/aws-apigateway";
import {Queue} from 'aws-cdk-lib/aws-sqs';
import {Topic} from "aws-cdk-lib/aws-sns";
import {SqsSubscription} from "aws-cdk-lib/aws-sns-subscriptions";
import {EventSourceMapping} from "aws-cdk-lib/aws-lambda";
import {PolicyStatement} from "aws-cdk-lib/aws-iam";

export class MotorshopCdkStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        /**
         * FACTORY
         */
            // https://github.com/aws-samples/serverless-patterns/blob/main/sns-sqs-lambda-cdk/src/lib/sns-sqs-lambda-cdk.ts
        const factory = new Construct(this, "factory");

        const snsModelDesign = new Topic(factory, 'sns-model-design', {
            topicName: 'sns-model-design',
            displayName: 'Publish new Model Design',
        });

        const sqsProspectGeneratorDlq = new Queue(factory, 'sqs-prospect-generator-dlq', {
            queueName: 'sqs-prospect-generator-dlq',
            removalPolicy: RemovalPolicy.DESTROY,
            retentionPeriod: cdk.Duration.days(1),

        });

        const sqsProspectGenerator = new Queue(factory, 'sqs-prospect-generator', {
            queueName: 'sqs-prospect-generator',
            removalPolicy: RemovalPolicy.DESTROY,
            deadLetterQueue: {
                maxReceiveCount: 3,
                queue: sqsProspectGeneratorDlq
            }
        });

        snsModelDesign.addSubscription(new SqsSubscription(sqsProspectGenerator, {
            rawMessageDelivery: true
        }));

        const bucket = new s3.Bucket(factory, 'bindstone-motorshop-bucket',
            {
                bucketName: 'bindstone-motorshop-bucket',
                removalPolicy: RemovalPolicy.DESTROY,
                autoDeleteObjects: true,
            });

        let publishModelLambda = new RustFunction(factory, 'motorshop-publish-model-lambda', {
            functionName: 'motorshop-publish-model-lambda',
            directory: '../',
            package: 'motorshop-publish-model-lambda',
            memorySize: 128,
            setupLogging: true,
            environment: {
                REBUILD_VERSION: "1",
                SNS_TOPIC_ARN: snsModelDesign.topicArn,
                SNS_TOPIC_NAME: snsModelDesign.topicName,
                RUST_BACKTRACE: "1",
                RUST_LOG: "info",
            },
        });
        snsModelDesign.grantPublish(publishModelLambda);

        let createProspectLambda = new RustFunction(factory, 'motorshop-create-prospect-lambda', {
            functionName: 'motorshop-create-prospect-lambda',
            directory: '../',
            package: 'motorshop-create-prospect-lambda',
            memorySize: 128,
            setupLogging: true,
            environment: {
                REBUILD_VERSION: "1",
                BUCKET_NAME: bucket.bucketName,
                RUST_BACKTRACE: "1",
                RUST_LOG: "info",
            },
        });

        createProspectLambda.addToRolePolicy(new PolicyStatement({
            actions: [
                'sqs:GetQueueAttributes',
                'sqs:ReceiveMessage',
                'sqs:DeleteMessage',
            ],
            resources: [
                sqsProspectGenerator.queueArn,
            ],
        }));

        const consumerEventSourceMapping = new EventSourceMapping(
            factory,
            'queue-consumer-create-lambda',
            {
                target: createProspectLambda,
                batchSize: 1,
                eventSourceArn: sqsProspectGenerator.queueArn,
                enabled: true,
            }
        );

        sqsProspectGenerator.grant(createProspectLambda);

        let orderLambda = new RustFunction(factory, 'motorshop-order-lambda', {
            functionName: 'motorshop-order-lambda',
            directory: '../',
            package: 'motorshop-order-lambda',
            memorySize: 128,
            setupLogging: true,
            environment: {
                RUST_BACKTRACE: "1",
                RUST_LOG: "info",
                REBUILD_VERSION: "1",
            },
        });

        let publishModelApi = new LambdaRestApi(factory, 'motorshop-publish-model-api', {
            handler: publishModelLambda,
            restApiName: 'motorshop-publish-model-api',
            endpointConfiguration: {types: [EndpointType.REGIONAL]},
        });

        // AUTHORISATIONS
        // @ts-ignore
        const grant = bucket.grantReadWrite(createProspectLambda);

        /**
         * SHOP
         */
        const shop = new Construct(this, "shop");

        let orderApi = new LambdaRestApi(shop, 'motorshop-order-lambda-api', {
            handler: orderLambda,
            restApiName: 'motorshop-order-api',
            endpointConfiguration: {types: [EndpointType.REGIONAL]},
        });
    }
}
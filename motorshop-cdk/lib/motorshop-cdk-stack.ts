import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import * as s3 from 'aws-cdk-lib/aws-s3'
import {RemovalPolicy} from "aws-cdk-lib/core";
import {RustFunction} from "rust.aws-cdk-lambda";
import {EndpointType, LambdaRestApi} from "aws-cdk-lib/aws-apigateway";

export class MotorshopCdkStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);
        const factory = new Construct(this, "factory");
        const bucket = new s3.Bucket(factory, 'bindstone-motorshop-bucket',
            {
                bucketName: 'bindstone-motorshop-bucket',
                removalPolicy: RemovalPolicy.DESTROY,
                autoDeleteObjects: true,
            });

        let orderLambda = new RustFunction(factory, 'motorshop-order-lambda', {
            functionName: 'motorshop-order-lambda',
            directory: '../motorshop-order-lambda/',
            memorySize: 128,
            setupLogging: true,
            environment: {
                BUCKET_NAME: bucket.bucketName,
            },
        });

        /** Level 3 not for RUST **/
        let orderApi = new LambdaRestApi(factory, 'motorshop-order-lambda-api', {
            // @ts-ignore
            handler: orderLambda,
            restApiName: 'motorshop-order-lambda-api',
            endpointConfiguration: {types: [EndpointType.REGIONAL]},
        });

        const shop = new Construct(this, "shop");

    }
}
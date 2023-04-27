#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import {MotorshopCdkStack} from '../lib/motorshop-cdk-stack';

const app = new cdk.App();
new MotorshopCdkStack(app, 'MotorshopCdkStack', {
    stackName: 'MotorshopCdkStack',
    description: 'Motor Shop',
    tags: {'MS': 'GLOBAL'},
    env: {account: process.env.CDK_DEFAULT_ACCOUNT, region: process.env.CDK_DEFAULT_REGION}
});
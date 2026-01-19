#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== S3-DynamoDB Sync Lambda Setup ===${NC}"

# Check required env vars
if [ -z "$AWS_REGION" ]; then
    echo -e "${RED}ERROR: AWS_REGION not set${NC}"
    exit 1
fi

if [ -z "$AWS_PROFILE" ]; then
    echo -e "${YELLOW}WARNING: AWS_PROFILE not set, using default${NC}"
fi

# Configuration - customize these
PROJECT_NAME="s3-dynamo-sync"
S3_BUCKET_NAME="${PROJECT_NAME}-bucket-$(aws sts get-caller-identity --query Account --output text)"
DYNAMODB_TABLE_NAME="${PROJECT_NAME}-table"
LAMBDA_ROLE_NAME="${PROJECT_NAME}-lambda-role"

echo -e "${YELLOW}Configuration:${NC}"
echo "  Region: $AWS_REGION"
echo "  S3 Bucket: $S3_BUCKET_NAME"
echo "  DynamoDB Table: $DYNAMODB_TABLE_NAME"
echo ""

# -------------------------------------------------------------------
# 1. Create S3 Bucket
# -------------------------------------------------------------------
echo -e "${GREEN}[1/5] Creating S3 Bucket...${NC}"

if aws s3api head-bucket --bucket "$S3_BUCKET_NAME" 2>/dev/null; then
    echo "  Bucket '$S3_BUCKET_NAME' already exists"
else
    # Note: LocationConstraint not needed for us-east-1
    if [ "$AWS_REGION" = "us-east-1" ]; then
        aws s3api create-bucket \
            --bucket "$S3_BUCKET_NAME"
    else
        aws s3api create-bucket \
            --bucket "$S3_BUCKET_NAME" \
            --create-bucket-configuration LocationConstraint="$AWS_REGION"
    fi
    echo -e "  ${GREEN}✓ Bucket created${NC}"
fi

# Enable versioning (optional but recommended)
aws s3api put-bucket-versioning \
    --bucket "$S3_BUCKET_NAME" \
    --versioning-configuration Status=Enabled
echo "  Versioning enabled"

# -------------------------------------------------------------------
# 2. Create DynamoDB Table
# -------------------------------------------------------------------
echo -e "${GREEN}[2/5] Creating DynamoDB Table...${NC}"

if aws dynamodb describe-table --table-name "$DYNAMODB_TABLE_NAME" 2>/dev/null; then
    echo "  Table '$DYNAMODB_TABLE_NAME' already exists"
else
    aws dynamodb create-table \
        --table-name "$DYNAMODB_TABLE_NAME" \
        --attribute-definitions \
            AttributeName=pk,AttributeType=S \
            AttributeName=sk,AttributeType=S \
        --key-schema \
            AttributeName=pk,KeyType=HASH \
            AttributeName=sk,KeyType=RANGE \
        --billing-mode PAY_PER_REQUEST \
        --tags Key=Project,Value="$PROJECT_NAME"
    
    echo "  Waiting for table to be active..."
    aws dynamodb wait table-exists --table-name "$DYNAMODB_TABLE_NAME"
    echo -e "  ${GREEN}✓ Table created${NC}"
fi

# -------------------------------------------------------------------
# 3. Create IAM Role for Lambda
# -------------------------------------------------------------------
echo -e "${GREEN}[3/5] Creating IAM Role for Lambda...${NC}"

TRUST_POLICY='{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Service": "lambda.amazonaws.com"
            },
            "Action": "sts:AssumeRole"
        }
    ]
}'

if aws iam get-role --role-name "$LAMBDA_ROLE_NAME" 2>/dev/null; then
    echo "  Role '$LAMBDA_ROLE_NAME' already exists"
else
    aws iam create-role \
        --role-name "$LAMBDA_ROLE_NAME" \
        --assume-role-policy-document "$TRUST_POLICY"
    echo -e "  ${GREEN}✓ Role created${NC}"
fi

# -------------------------------------------------------------------
# 4. Attach Policies to Role
# -------------------------------------------------------------------
echo -e "${GREEN}[4/5] Attaching Policies...${NC}"

# Basic Lambda execution
aws iam attach-role-policy \
    --role-name "$LAMBDA_ROLE_NAME" \
    --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole 2>/dev/null || true
echo "  ✓ Lambda basic execution policy"

# Create custom policy for S3 and DynamoDB access
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

CUSTOM_POLICY=$(cat <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "s3:GetObject",
                "s3:PutObject",
                "s3:DeleteObject",
                "s3:ListBucket",
                "s3:GetObjectVersion"
            ],
            "Resource": [
                "arn:aws:s3:::${S3_BUCKET_NAME}",
                "arn:aws:s3:::${S3_BUCKET_NAME}/*"
            ]
        },
        {
            "Effect": "Allow",
            "Action": [
                "dynamodb:PutItem",
                "dynamodb:GetItem",
                "dynamodb:UpdateItem",
                "dynamodb:DeleteItem",
                "dynamodb:Query",
                "dynamodb:Scan"
            ],
            "Resource": "arn:aws:dynamodb:${AWS_REGION}:${ACCOUNT_ID}:table/${DYNAMODB_TABLE_NAME}"
        }
    ]
}
EOF
)

POLICY_NAME="${PROJECT_NAME}-policy"

# Check if policy exists and delete it first (to update)
if aws iam get-role-policy --role-name "$LAMBDA_ROLE_NAME" --policy-name "$POLICY_NAME" 2>/dev/null; then
    aws iam delete-role-policy --role-name "$LAMBDA_ROLE_NAME" --policy-name "$POLICY_NAME"
fi

aws iam put-role-policy \
    --role-name "$LAMBDA_ROLE_NAME" \
    --policy-name "$POLICY_NAME" \
    --policy-document "$CUSTOM_POLICY"
echo "  ✓ Custom S3/DynamoDB policy attached"

# -------------------------------------------------------------------
# 5. Set up S3 Event Notification (for Lambda trigger)
# -------------------------------------------------------------------
echo -e "${GREEN}[5/5] S3 Event Notification will be configured after Lambda deployment${NC}"

# -------------------------------------------------------------------
# Output configuration for the Rust project
# -------------------------------------------------------------------
echo ""
echo -e "${GREEN}=== Setup Complete ===${NC}"
echo ""
echo -e "${YELLOW}Save these values for your Lambda configuration:${NC}"
echo "export S3_BUCKET_NAME=\"$S3_BUCKET_NAME\""
echo "export DYNAMODB_TABLE_NAME=\"$DYNAMODB_TABLE_NAME\""
echo "export LAMBDA_ROLE_ARN=\"arn:aws:iam::${ACCOUNT_ID}:role/${LAMBDA_ROLE_NAME}\""
echo ""

# Save to .env file for convenience
cat > .env <<EOF
S3_BUCKET_NAME=$S3_BUCKET_NAME
DYNAMODB_TABLE_NAME=$DYNAMODB_TABLE_NAME
LAMBDA_ROLE_ARN=arn:aws:iam::${ACCOUNT_ID}:role/${LAMBDA_ROLE_NAME}
AWS_REGION=$AWS_REGION
EOF
echo -e "${GREEN}Configuration saved to .env${NC}"

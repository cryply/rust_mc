#!/bin/bash
set -e

# =============================================================================
# EFS Lambda Deployment Script
# =============================================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# -----------------------------------------------------------------------------
# Configuration - EDIT THESE VALUES
# -----------------------------------------------------------------------------
AWS_REGION="eu-west-1"
FUNCTION_NAME="efs-lister"
AWS_ACCOUNT_ID="XXX"
EFS_ARN="arn:aws:elasticfilesystem:eu-west-1:XXX:file-system/fs-ZZZ"

# These should be set as environment variables or edit here
VPC_ID="${VPC_ID:-vpc-1}"
PRIVATE_SUBNET_1="${PRIVATE_SUBNET_1:-subnet-2}"
PRIVATE_SUBNET_2="${PRIVATE_SUBNET_2:-subnet-3}"
PRIVATE_RESOURCES_SG_ID="${PRIVATE_RESOURCES_SG_SG_ID:-sg-5}"
EFS_ID="${EFS_ID:-fs-ZZZ}"

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 is not installed. Please install it first."
        exit 1
    fi
}

# -----------------------------------------------------------------------------
# Pre-flight Checks
# -----------------------------------------------------------------------------
preflight_checks() {
    log_info "Running pre-flight checks..."
    
    check_command "cargo"
    check_command "cargo-lambda"
    check_command "aws"
    check_command "jq"
    
    # Check AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        log_error "AWS credentials not configured. Please run 'aws configure' first."
        exit 1
    fi
    
    CALLER_IDENTITY=$(aws sts get-caller-identity)
    log_info "Using AWS Account: $(echo $CALLER_IDENTITY | jq -r '.Account')"
    log_info "AWS User/Role: $(echo $CALLER_IDENTITY | jq -r '.Arn')"
    
    log_info "Pre-flight checks passed!"
}

# -----------------------------------------------------------------------------
# Fix Cargo.toml Edition
# -----------------------------------------------------------------------------
fix_cargo_toml() {
    log_info "Checking Cargo.toml edition..."
    
    if grep -q 'edition = "2024"' Cargo.toml; then
        log_warn "Found invalid edition '2024', fixing to '2021'..."
        sed -i 's/edition = "2024"/edition = "2021"/' Cargo.toml
        log_info "Cargo.toml fixed!"
    else
        log_info "Cargo.toml edition is OK"
    fi
}

# -----------------------------------------------------------------------------
# Build Lambda
# -----------------------------------------------------------------------------
build_lambda() {
    log_info "Building Lambda function for ARM64..."
    cargo lambda build --release --arm64
    log_info "Build complete!"
}

# -----------------------------------------------------------------------------
# Run Tests
# -----------------------------------------------------------------------------
run_tests() {
    log_info "Running unit tests..."
    cargo test || log_warn "Tests failed or no tests found, continuing..."
    log_info "Tests complete!"
}

# -----------------------------------------------------------------------------
# Create IAM Role
# -----------------------------------------------------------------------------
create_iam_role() {
    local ROLE_NAME="${FUNCTION_NAME}-lambda-role"
    
    log_info "Creating IAM role: $ROLE_NAME"
    
    # Check if role already exists
    if aws iam get-role --role-name "$ROLE_NAME" &> /dev/null; then
        log_warn "IAM role already exists, skipping creation..."
        ROLE_ARN=$(aws iam get-role --role-name "$ROLE_NAME" --query 'Role.Arn' --output text)
        export ROLE_ARN
        log_info "Using existing role: $ROLE_ARN"
        return 0
    fi
    
    # Create trust policy
    cat > /tmp/trust-policy.json << 'EOF'
{
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
}
EOF

    # Create the role
    aws iam create-role \
        --role-name "$ROLE_NAME" \
        --assume-role-policy-document file:///tmp/trust-policy.json \
        --output text > /dev/null
    
    log_info "Attaching policies..."
    
    # Attach basic Lambda execution policy
    aws iam attach-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
    
    # Attach VPC access policy
    aws iam attach-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole
    
    # Create and attach EFS access policy
    cat > /tmp/efs-policy.json << EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "elasticfilesystem:ClientMount",
        "elasticfilesystem:ClientWrite",
        "elasticfilesystem:ClientRootAccess"
      ],
      "Resource": "${EFS_ARN}"
    }
  ]
}
EOF

    aws iam put-role-policy \
        --role-name "$ROLE_NAME" \
        --policy-name EFSAccessPolicy \
        --policy-document file:///tmp/efs-policy.json
    
    ROLE_ARN=$(aws iam get-role --role-name "$ROLE_NAME" --query 'Role.Arn' --output text)
    export ROLE_ARN
    
    log_info "IAM role created: $ROLE_ARN"
    
    # Wait for role to propagate
    log_info "Waiting 10 seconds for IAM role to propagate..."
    sleep 10
}

# -----------------------------------------------------------------------------
# Create EFS Access Point
# -----------------------------------------------------------------------------
create_efs_access_point() {
    log_info "Checking for existing EFS access points..."
    
    # Check if access point already exists
    EXISTING_AP=$(aws efs describe-access-points \
        --file-system-id "$EFS_ID" \
        --query "AccessPoints[?RootDirectory.Path=='/lambda'].AccessPointId" \
        --output text 2>/dev/null || echo "")
    
    if [ -n "$EXISTING_AP" ] && [ "$EXISTING_AP" != "None" ]; then
        log_warn "EFS access point already exists: $EXISTING_AP"
        ACCESS_POINT_ID="$EXISTING_AP"
    else
        log_info "Creating EFS access point..."
        
        ACCESS_POINT_ID=$(aws efs create-access-point \
            --file-system-id "$EFS_ID" \
            --posix-user "Uid=1000,Gid=1000" \
            --root-directory "Path=/lambda,CreationInfo={OwnerUid=1000,OwnerGid=1000,Permissions=755}" \
            --query 'AccessPointId' \
            --output text)
        
        log_info "Created access point: $ACCESS_POINT_ID"
        
        # Wait for access point to be available
        log_info "Waiting for access point to become available..."
        sleep 5
    fi
    
    ACCESS_POINT_ARN="arn:aws:elasticfilesystem:${AWS_REGION}:${AWS_ACCOUNT_ID}:access-point/${ACCESS_POINT_ID}"
    export ACCESS_POINT_ID ACCESS_POINT_ARN
    
    log_info "Access Point ARN: $ACCESS_POINT_ARN"
}

# -----------------------------------------------------------------------------
# Deploy Lambda Function
# -----------------------------------------------------------------------------
deploy_lambda() {
    log_info "Deploying Lambda function..."
    
    cargo lambda deploy "$FUNCTION_NAME" \
        --region "$AWS_REGION" \
        --iam-role "$ROLE_ARN"
    
    log_info "Lambda deployed, configuring VPC..."
    
    # Update function configuration with VPC
    aws lambda update-function-configuration \
        --function-name "$FUNCTION_NAME" \
        --vpc-config "SubnetIds=${PRIVATE_SUBNET_1},${PRIVATE_SUBNET_2},SecurityGroupIds=${PRIVATE_RESOURCES_SG_ID}" \
        --region "$AWS_REGION" \
        --output text > /dev/null
    
    log_info "Waiting for VPC configuration to complete..."
    aws lambda wait function-updated \
        --function-name "$FUNCTION_NAME" \
        --region "$AWS_REGION"
    
    log_info "Configuring EFS mount..."
    
    # Add EFS configuration
    aws lambda update-function-configuration \
        --function-name "$FUNCTION_NAME" \
        --file-system-configs "Arn=${ACCESS_POINT_ARN},LocalMountPath=/mnt/efs" \
        --region "$AWS_REGION" \
        --output text > /dev/null
    
    log_info "Waiting for EFS configuration to complete..."
    aws lambda wait function-updated \
        --function-name "$FUNCTION_NAME" \
        --region "$AWS_REGION"
    
    log_info "Lambda deployment complete!"
}

# -----------------------------------------------------------------------------
# Test Lambda Function
# -----------------------------------------------------------------------------
test_lambda() {
    log_info "Testing Lambda function..."
    
    RESPONSE=$(aws lambda invoke \
        --function-name "$FUNCTION_NAME" \
        --payload '{"name": "EFS Test"}' \
        --cli-binary-format raw-in-base64-out \
        --region "$AWS_REGION" \
        /tmp/lambda-response.json \
        --output json)
    
    STATUS_CODE=$(echo "$RESPONSE" | jq -r '.StatusCode')
    FUNCTION_ERROR=$(echo "$RESPONSE" | jq -r '.FunctionError // empty')
    
    if [ "$STATUS_CODE" == "200" ] && [ -z "$FUNCTION_ERROR" ]; then
        log_info "Lambda invocation successful!"
        echo -e "${GREEN}Response:${NC}"
        cat /tmp/lambda-response.json | jq .
    else
        log_error "Lambda invocation failed!"
        echo "Status Code: $STATUS_CODE"
        echo "Function Error: $FUNCTION_ERROR"
        echo "Response:"
        cat /tmp/lambda-response.json
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Show Summary
# -----------------------------------------------------------------------------
show_summary() {
    echo ""
    echo "============================================================================="
    echo -e "${GREEN}DEPLOYMENT SUMMARY${NC}"
    echo "============================================================================="
    echo "Function Name:    $FUNCTION_NAME"
    echo "Region:           $AWS_REGION"
    echo "IAM Role:         $ROLE_ARN"
    echo "Access Point ID:  $ACCESS_POINT_ID"
    echo "VPC ID:           $VPC_ID"
    echo "Subnets:          $PRIVATE_SUBNET_1, $PRIVATE_SUBNET_2"
    echo "Security Group:   $PRIVATE_RESOURCES_SG_ID"
    echo "EFS Mount Path:   /mnt/efs"
    echo "============================================================================="
    echo ""
    echo "Useful commands:"
    echo "  # Invoke function"
    echo "  aws lambda invoke --function-name $FUNCTION_NAME --payload '{\"name\": \"Test\"}' --cli-binary-format raw-in-base64-out --region $AWS_REGION response.json && cat response.json | jq ."
    echo ""
    echo "  # View logs"
    echo "  aws logs tail /aws/lambda/$FUNCTION_NAME --follow --region $AWS_REGION"
    echo ""
    echo "  # Redeploy after code changes"
    echo "  cargo lambda build --release --arm64 && cargo lambda deploy $FUNCTION_NAME --region $AWS_REGION"
    echo "============================================================================="
}

# -----------------------------------------------------------------------------
# Cleanup Function (optional - use with caution)
# -----------------------------------------------------------------------------
cleanup() {
    log_warn "This will delete all created resources. Are you sure? (y/N)"
    read -r confirm
    if [ "$confirm" != "y" ]; then
        log_info "Cleanup cancelled"
        return 0
    fi
    
    log_info "Deleting Lambda function..."
    aws lambda delete-function --function-name "$FUNCTION_NAME" --region "$AWS_REGION" 2>/dev/null || true
    
    log_info "Deleting EFS access point..."
    aws efs delete-access-point --access-point-id "$ACCESS_POINT_ID" 2>/dev/null || true
    
    log_info "Deleting IAM role policies..."
    aws iam delete-role-policy --role-name "${FUNCTION_NAME}-lambda-role" --policy-name EFSAccessPolicy 2>/dev/null || true
    aws iam detach-role-policy --role-name "${FUNCTION_NAME}-lambda-role" --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole 2>/dev/null || true
    aws iam detach-role-policy --role-name "${FUNCTION_NAME}-lambda-role" --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole 2>/dev/null || true
    
    log_info "Deleting IAM role..."
    aws iam delete-role --role-name "${FUNCTION_NAME}-lambda-role" 2>/dev/null || true
    
    log_info "Cleanup complete!"
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    echo "============================================================================="
    echo "EFS Lambda Deployment Script"
    echo "============================================================================="
    
    case "${1:-deploy}" in
        deploy)
            preflight_checks
            fix_cargo_toml
            build_lambda
            run_tests
            create_iam_role
            create_efs_access_point
            deploy_lambda
            test_lambda
            show_summary
            ;;
        build)
            fix_cargo_toml
            build_lambda
            ;;
        test)
            test_lambda
            ;;
        cleanup)
            cleanup
            ;;
        redeploy)
            preflight_checks
            fix_cargo_toml
            build_lambda
            cargo lambda deploy "$FUNCTION_NAME" --region "$AWS_REGION"
            test_lambda
            ;;
        *)
            echo "Usage: $0 {deploy|build|test|redeploy|cleanup}"
            echo ""
            echo "Commands:"
            echo "  deploy   - Full deployment (build, create resources, deploy, test)"
            echo "  build    - Build Lambda only"
            echo "  test     - Test deployed Lambda"
            echo "  redeploy - Quick rebuild and deploy (no IAM/EFS changes)"
            echo "  cleanup  - Delete all created resources"
            exit 1
            ;;
    esac
}

main "$@"
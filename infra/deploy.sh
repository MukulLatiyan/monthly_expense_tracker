#!/usr/bin/env bash
# Deploy CloudFormation (S3 + CloudFront) then build and upload the React app.
#
# Usage:
#   ./infra/deploy.sh [prod|dev]
#
# Prerequisites:
#   - AWS CLI configured
#   - VITE_API_BASE_URL set in the environment OR in expense-tracker-fe/.env.production
#     (must be your API Gateway base URL, e.g. https://xxx.execute-api.ap-south-1.amazonaws.com/prod)
#
# Optional custom domain (set env vars before running):
#   DOMAIN_NAME, ACM_CERTIFICATE_ARN, HOSTED_ZONE_ID
# Or run ./infra/bootstrap-domain.sh first and paste values.

set -euo pipefail

ENV="${1:-prod}"
REGION="${AWS_DEFAULT_REGION:-ap-south-1}"
STACK_NAME="expense-tracker-frontend-${ENV}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FE_DIR="${ROOT_DIR}/expense-tracker-fe"

cd "${ROOT_DIR}"

DOMAIN_NAME="${DOMAIN_NAME:-}"
ACM_CERTIFICATE_ARN="${ACM_CERTIFICATE_ARN:-}"
HOSTED_ZONE_ID="${HOSTED_ZONE_ID:-}"

if [[ -n "${DOMAIN_NAME}" || -n "${ACM_CERTIFICATE_ARN}" || -n "${HOSTED_ZONE_ID}" ]]; then
  if [[ -z "${DOMAIN_NAME}" || -z "${ACM_CERTIFICATE_ARN}" || -z "${HOSTED_ZONE_ID}" ]]; then
    echo "For a custom domain, set all three: DOMAIN_NAME, ACM_CERTIFICATE_ARN, HOSTED_ZONE_ID." >&2
    echo "Or leave all unset to use the default CloudFront URL (and keep existing domain on stack updates)." >&2
    exit 1
  fi
fi

echo "==> Deploying stack ${STACK_NAME} (${REGION})..."

if [[ -n "${DOMAIN_NAME}" ]]; then
  aws cloudformation deploy \
    --template-file "${ROOT_DIR}/infra/template.yaml" \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --parameter-overrides \
      "Environment=${ENV}" \
      "DomainName=${DOMAIN_NAME}" \
      "AcmCertificateArn=${ACM_CERTIFICATE_ARN}" \
      "HostedZoneId=${HOSTED_ZONE_ID}"
else
  aws cloudformation deploy \
    --template-file "${ROOT_DIR}/infra/template.yaml" \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --parameter-overrides "Environment=${ENV}"
fi

get_output() {
  aws cloudformation describe-stacks \
    --stack-name "${STACK_NAME}" \
    --region "${REGION}" \
    --query "Stacks[0].Outputs[?OutputKey=='${1}'].OutputValue" \
    --output text
}

BUCKET=$(get_output "FrontendBucketName")
CF_ID=$(get_output "CloudFrontDistributionId")
APP_URL=$(get_output "CloudFrontURL")

echo "  Bucket:     ${BUCKET}"
echo "  CloudFront: ${APP_URL}"
echo ""

if [[ ! -d "${FE_DIR}" ]]; then
  echo "Missing frontend at ${FE_DIR}" >&2
  exit 1
fi

# Load API URL for Vite build
if [[ -f "${FE_DIR}/.env.production" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${FE_DIR}/.env.production"
  set +a
fi

if [[ -z "${VITE_API_BASE_URL:-}" ]]; then
  echo "Set VITE_API_BASE_URL (export or expense-tracker-fe/.env.production) to your API Gateway base URL." >&2
  exit 1
fi

echo "==> Building frontend (VITE_API_BASE_URL=${VITE_API_BASE_URL})..."
cd "${FE_DIR}"
npm ci --silent
npm run build

echo ""
echo "==> Syncing to s3://${BUCKET}/ ..."
aws s3 sync dist/ "s3://${BUCKET}/" \
  --delete \
  --region "${REGION}" \
  --exclude "index.html" \
  --cache-control "public, max-age=31536000, immutable"

aws s3 cp dist/index.html "s3://${BUCKET}/index.html" \
  --region "${REGION}" \
  --cache-control "no-cache, no-store, must-revalidate" \
  --content-type "text/html; charset=utf-8"

echo ""
echo "==> Invalidating CloudFront cache (${CF_ID})..."
aws cloudfront create-invalidation \
  --distribution-id "${CF_ID}" \
  --paths "/*" \
  --region us-east-1 \
  --output text \
  --query "Invalidation.Id" | xargs -I{} echo "  Invalidation ID: {}"

echo ""
echo "════════════════════════════════════════════════════════════"
echo " ✓ Deploy complete"
echo "   App:    ${APP_URL}"
echo "   Bucket: s3://${BUCKET}"
echo "════════════════════════════════════════════════════════════"

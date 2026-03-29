#!/usr/bin/env bash
# One-time bootstrap: Route 53 hosted zone check, ACM cert in us-east-1, DNS validation,
# prints values to use with infra/deploy.sh for a custom domain on CloudFront.
#
# Usage:
#   ./infra/bootstrap-domain.sh yourdomain.com
#
# Prerequisites:
#   - AWS CLI with Route 53 + ACM permissions
#   - Domain registered (ideally in Route 53) for the zone you pass

set -euo pipefail

DOMAIN="${1:-}"
if [[ -z "${DOMAIN}" ]]; then
  echo "Usage: $0 <domain.com>" >&2
  exit 1
fi

REGION="${AWS_DEFAULT_REGION:-ap-south-1}"

echo "==> Domain bootstrap for ${DOMAIN}"
echo ""

echo "── Step 1: Route 53 hosted zone"
ZONE_ID=$(aws route53 list-hosted-zones-by-name \
  --dns-name "${DOMAIN}." \
  --query "HostedZones[?Name=='${DOMAIN}.'].Id" \
  --output text 2>/dev/null | sed 's|/hostedzone/||' || true)

if [[ -z "${ZONE_ID}" ]]; then
  echo "  No hosted zone for ${DOMAIN}. — create one in Route 53 or register the domain first."
  echo "  To create:"
  echo "    aws route53 create-hosted-zone --name ${DOMAIN} --caller-reference \$(date +%s)"
  exit 1
fi
echo "  ✓ Hosted zone: ${ZONE_ID}"
echo ""

echo "── Step 2: ACM certificate (us-east-1, required for CloudFront)"
EXISTING_CERT=$(aws acm list-certificates \
  --region us-east-1 \
  --query "CertificateSummaryList[?DomainName=='${DOMAIN}' || DomainName=='*.${DOMAIN}'].CertificateArn | [0]" \
  --output text 2>/dev/null || echo "")

if [[ -n "${EXISTING_CERT}" && "${EXISTING_CERT}" != "None" ]]; then
  CERT_ARN="${EXISTING_CERT}"
  echo "  ✓ Using existing certificate: ${CERT_ARN}"
else
  echo "  Requesting cert for ${DOMAIN} and *.${DOMAIN}..."
  CERT_ARN=$(aws acm request-certificate \
    --region us-east-1 \
    --domain-name "*.${DOMAIN}" \
    --subject-alternative-names "${DOMAIN}" \
    --validation-method DNS \
    --query "CertificateArn" \
    --output text)
  echo "  ✓ Requested: ${CERT_ARN}"

  echo "  Creating DNS validation records..."
  sleep 5
  VALIDATION_OPTIONS=$(aws acm describe-certificate \
    --region us-east-1 \
    --certificate-arn "${CERT_ARN}" \
    --query "Certificate.DomainValidationOptions[*].ResourceRecord" \
    --output json)

  CNAME_NAME=$(echo "${VALIDATION_OPTIONS}" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d[0]['Name'])")
  CNAME_VALUE=$(echo "${VALIDATION_OPTIONS}" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d[0]['Value'])")

  aws route53 change-resource-record-sets \
    --hosted-zone-id "${ZONE_ID}" \
    --change-batch "{
      \"Changes\": [{
        \"Action\": \"UPSERT\",
        \"ResourceRecordSet\": {
          \"Name\": \"${CNAME_NAME}\",
          \"Type\": \"CNAME\",
          \"TTL\": 300,
          \"ResourceRecords\": [{\"Value\": \"${CNAME_VALUE}\"}]
        }
      }]
    }" >/dev/null

  echo "  Waiting for certificate validation (can take a few minutes)..."
  aws acm wait certificate-validated --region us-east-1 --certificate-arn "${CERT_ARN}"
  echo "  ✓ Certificate validated"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo " Use these when deploying with a custom domain:"
echo ""
echo "  export DOMAIN_NAME=${DOMAIN}"
echo "  export ACM_CERTIFICATE_ARN=${CERT_ARN}"
echo "  export HOSTED_ZONE_ID=${ZONE_ID}"
echo "  ./infra/deploy.sh prod"
echo ""
echo " Or add to expense-tracker-fe/.env.production (API URL only);"
echo " pass domain vars on the command line as above."
echo "════════════════════════════════════════════════════════════"

#!/usr/bin/env bash
# The static frontend stack (S3 + CloudFront) does not use SSM secrets.
# Backend secrets (if any) are configured with expense-tracker-be deployment.
#
# Usage: ./infra/setup-secrets.sh [prod|dev]

set -euo pipefail

echo "No SSM parameters required for infra/template.yaml (frontend only)."
echo "Deploy the stack with: ./infra/deploy.sh \"\${1:-prod}\""
exit 0

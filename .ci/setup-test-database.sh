#!/bin/bash
set -euo pipefail
cat samples/db-setup.sql samples/user-setup.sql .ci/test-data.sql \
    | envsubst \
    | psql -b1v ON_ERROR_STOP=1

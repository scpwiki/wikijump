#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_ROOT="$(mktemp -d /tmp/wikidot-python-setup-test.XXXXXX)"
trap 'rm -rf -- "${FIXTURE_ROOT}"' EXIT

mkdir -p "${FIXTURE_ROOT}/project/scripts" "${FIXTURE_ROOT}/bin"
cp "${PROJECT_ROOT}/scripts/setup-python-env.sh" "${FIXTURE_ROOT}/project/scripts/"
cp "${PROJECT_ROOT}/requirements.lock" "${FIXTURE_ROOT}/project/"

cat >"${FIXTURE_ROOT}/bin/python3" <<'PYTHON'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == "-m" && "$2" == "venv" && -n "$3" ]]
mkdir -p "$3/bin"
cat >"$3/bin/python" <<'VENV_PYTHON'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >"${SETUP_TEST_LOG}"
VENV_PYTHON
chmod +x "$3/bin/python"
PYTHON
chmod +x "${FIXTURE_ROOT}/bin/python3"

SETUP_TEST_LOG="${FIXTURE_ROOT}/pip-arguments" PATH="${FIXTURE_ROOT}/bin:${PATH}" "${FIXTURE_ROOT}/project/scripts/setup-python-env.sh"
EXPECTED="-m pip install --requirement ${FIXTURE_ROOT}/project/requirements.lock"
[[ "$(<"${FIXTURE_ROOT}/pip-arguments")" == "${EXPECTED}" ]]

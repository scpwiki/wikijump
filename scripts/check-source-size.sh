#!/usr/bin/env bash

set -euo pipefail

readonly warning_limit=1500
readonly severe_limit=2500
readonly baseline_file="scripts/source-size-baseline.txt"
readonly base_sha="${SOURCE_SIZE_BASE_SHA:-}"

declare -A severe_baseline=()
declare -A base_severe_baseline=()

while read -r ceiling path; do
    if [[ -z "${ceiling}" || "${ceiling}" == \#* ]]; then
        continue
    fi
    if [[ ! "${ceiling}" =~ ^[0-9]+$ || -z "${path}" ]]; then
        echo "Invalid source-size baseline entry: ${ceiling} ${path}" >&2
        exit 1
    fi
    if [[ -n "${severe_baseline["${path}"]+present}" ]]; then
        echo "Duplicate source-size baseline entry: ${path}" >&2
        exit 1
    fi
    severe_baseline["${path}"]="${ceiling}"
done < "${baseline_file}"

if [[ -n "${base_sha}" ]]; then
    if ! git cat-file -e "${base_sha}^{commit}" 2>/dev/null; then
        echo "Source-size base commit is unavailable: ${base_sha}" >&2
        exit 1
    fi

    if git cat-file -e "${base_sha}:${baseline_file}" 2>/dev/null; then
        while read -r ceiling path; do
            if [[ -z "${ceiling}" || "${ceiling}" == \#* ]]; then
                continue
            fi
            if [[ ! "${ceiling}" =~ ^[0-9]+$ || -z "${path}" ]]; then
                echo "Invalid base source-size baseline entry: ${ceiling} ${path}" >&2
                exit 1
            fi
            base_severe_baseline["${path}"]="${ceiling}"
        done < <(git show "${base_sha}:${baseline_file}")

        for path in "${!severe_baseline[@]}"; do
            ceiling=${severe_baseline["${path}"]}
            base_ceiling=${base_severe_baseline["${path}"]:-}
            if [[ -z "${base_ceiling}" || "${ceiling}" -gt "${base_ceiling}" ]]; then
                echo "Source-size baseline additions and increases are not allowed: ${path} ${base_ceiling:-absent} -> ${ceiling}" >&2
                exit 1
            fi
        done
    else
        for path in "${!severe_baseline[@]}"; do
            ceiling=${severe_baseline["${path}"]}
            if ! git cat-file -e "${base_sha}:${path}" 2>/dev/null; then
                echo "Bootstrap source-size baseline path is absent from the base commit: ${path}" >&2
                exit 1
            fi
            base_lines=$(git show "${base_sha}:${path}" | wc -l)
            if (( base_lines <= severe_limit || ceiling != base_lines )); then
                echo "Bootstrap source-size baseline must exactly match a pre-existing severe file: ${path} has ${base_lines} base lines, candidate ceiling ${ceiling}" >&2
                exit 1
            fi
        done
    fi
fi

warning_count=0
severe_count=0

while IFS= read -r -d '' path; do
    case "${path}" in
        *.rs|*.ts|*.svelte) ;;
        *) continue ;;
    esac

    case "/${path}/" in
        */test/*|*/tests/*|*/vendor/*|*/generated/*|*/node_modules/*) continue ;;
    esac

    case "${path}" in
        */test.rs|*/tests.rs|*_test.rs|*.test.ts|*.spec.ts|*.generated.ts) continue ;;
    esac

    lines=$(wc -l < "${path}")
    if (( lines <= warning_limit )); then
        continue
    fi

    ((warning_count += 1))
    if (( lines <= severe_limit )); then
        echo "::warning file=${path}::${path} has ${lines} lines; the source-size warning limit is ${warning_limit}."
        continue
    fi

    ceiling=${severe_baseline["${path}"]:-}
    if [[ -n "${ceiling}" ]] && (( lines <= ceiling )); then
        echo "::warning file=${path}::${path} has ${lines} lines; this pre-existing severe violation is grandfathered up to ${ceiling} lines."
        continue
    fi

    ((severe_count += 1))
    if [[ -n "${ceiling}" ]]; then
        echo "::error file=${path}::${path} has grown to ${lines} lines, above its grandfathered ceiling of ${ceiling}."
    else
        echo "::error file=${path}::${path} has ${lines} lines; new severe violations above ${severe_limit} lines are not allowed."
    fi
done < <(git ls-files -z)

echo "Source-size budget: ${warning_count} file(s) above ${warning_limit} lines; ${severe_count} new or grown severe violation(s) above ${severe_limit} lines."

if (( severe_count > 0 )); then
    exit 1
fi

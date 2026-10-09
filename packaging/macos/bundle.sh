#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd -- "${script_dir}/../.." && pwd)"
profile="${1:-release}"

case "${profile}" in
    release)
        cargo_args=(--release)
        ;;
    debug)
        cargo_args=()
        ;;
    *)
        echo "usage: $0 [release|debug]" >&2
        exit 2
        ;;
esac

cd "${project_root}"
cargo build -p us-hr-control "${cargo_args[@]}"

app_dir="${project_root}/target/${profile}/US-HR Custom Control.app"
contents_dir="${app_dir}/Contents"
macos_dir="${contents_dir}/MacOS"
resources_dir="${contents_dir}/Resources"

rm -rf -- "${app_dir}"
install -d -- "${macos_dir}" "${resources_dir}"
install -m 0755 -- "${project_root}/target/${profile}/us-hr-control" \
    "${macos_dir}/us-hr-control"
install -m 0644 -- "${script_dir}/Info.plist" "${contents_dir}/Info.plist"
install -m 0644 -- "${script_dir}/PkgInfo" "${contents_dir}/PkgInfo"
install -m 0644 -- "${script_dir}/AppIcon.icns" \
    "${resources_dir}/AppIcon.icns"

codesign --force --sign - --timestamp=none "${app_dir}"
echo "Created ${app_dir}"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd -- "${script_dir}/../.." && pwd)"
source_icon="${project_root}/assets/app-icon.png"
output_icon="${script_dir}/AppIcon.icns"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/us-hr-app-icon.XXXXXX")"
iconset_dir="${work_dir}/AppIcon.iconset"

cleanup() {
    rm -rf -- "${work_dir}"
}
trap cleanup EXIT

mkdir -p "${iconset_dir}"

render_icon() {
    local size="$1"
    local filename="$2"
    sips -z "${size}" "${size}" "${source_icon}" \
        --out "${iconset_dir}/${filename}" >/dev/null
}

render_icon 16 icon_16x16.png
render_icon 32 icon_16x16@2x.png
render_icon 32 icon_32x32.png
render_icon 64 icon_32x32@2x.png
render_icon 128 icon_128x128.png
render_icon 256 icon_128x128@2x.png
render_icon 256 icon_256x256.png
render_icon 512 icon_256x256@2x.png
render_icon 512 icon_512x512.png
render_icon 1024 icon_512x512@2x.png

iconutil --convert icns --output "${output_icon}" "${iconset_dir}"
echo "Created ${output_icon}"

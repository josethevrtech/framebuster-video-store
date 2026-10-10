#!/usr/bin/env bash
set -euo pipefail

declare -A downloads=(
    [glyphew-uv-180.mp4]="https://thanford.com/glyphew/clips/uv_180.mp4"
    [glyphew-uv-180-tb.mp4]="https://thanford.com/glyphew/clips/uv_180_tb.mp4"
    [glyphew-uv-flat-tb.mp4]="https://thanford.com/glyphew/clips/uv_flat_tb.mp4"

    [canon-rf5.2mm-original.crm]="https://app.ssw.imaging-saas.canon/data/vr/A_0002C015X250315_235831EJ_CANON.CRM.CRM"
    [canon-rf5.2mm-finished.zip]="https://app.ssw.imaging-saas.canon/data/vr/RF5.2mmFinished_Clips.zip"
    [canon-rf-s3.9mm-original.mp4]="https://app.ssw.imaging-saas.canon/data/vr/A_0006C043H260222_211632NZ_CANON.MP4"
    [canon-rf-s3.9mm-finished.zip]="https://app.ssw.imaging-saas.canon/data/vr/RF-S3.9mmFinished_Clips.zip"
    [canon-rf-s7.8mm-original.mp4]="https://app.ssw.imaging-saas.canon/data/vr/A_0006C039H251103_110330IM_CANON.MP4"
    [canon-rf-s7.8mm-finished.zip]="https://app.ssw.imaging-saas.canon/data/vr/RF-S7.8mmFinished_Clips.zip"

    [remotion-video.mp4]="https://remotion.media/video.mp4"
    [remotion-video-flac.mp4]="https://remotion.media/video-flac.mp4"
    [remotion-video-mp3.mp4]="https://remotion.media/video-mp3.mp4"
    [remotion-video-pcm.mp4]="https://remotion.media/video-pcm.mp4"
    [remotion-video-none.mp4]="https://remotion.media/video-none.mp4"
    [remotion-video-h265.mp4]="https://remotion.media/video-h265.mp4"
    [remotion-video-h265-flac.mp4]="https://remotion.media/video-h265-flac.mp4"
    [remotion-video-h265-mp3.mp4]="https://remotion.media/video-h265-mp3.mp4"
    [remotion-video-h265-pcm.mp4]="https://remotion.media/video-h265-pcm.mp4"
    [remotion-video-h265-none.mp4]="https://remotion.media/video-h265-none.mp4"
    [remotion-video-opus.webm]="https://remotion.media/video-opus.webm"
    [remotion-video-vorbis.webm]="https://remotion.media/video-vorbis.webm"
    [remotion-video-none.webm]="https://remotion.media/video-none.webm"
    [remotion-video-vp9-opus.webm]="https://remotion.media/video-vp9-opus.webm"
    [remotion-video-vp9-vorbis.webm]="https://remotion.media/video-vp9-vorbis.webm"
    [remotion-video-vp9-none.webm]="https://remotion.media/video-vp9-none.webm"
    [remotion-video.mkv]="https://remotion.media/video.mkv"
    [remotion-video-flac.mkv]="https://remotion.media/video-flac.mkv"
    [remotion-video-mp3.mkv]="https://remotion.media/video-mp3.mkv"
    [remotion-video-opus.mkv]="https://remotion.media/video-opus.mkv"
    [remotion-video-vorbis.mkv]="https://remotion.media/video-vorbis.mkv"
    [remotion-video-none.mkv]="https://remotion.media/video-none.mkv"
    [remotion-video-h265.mkv]="https://remotion.media/video-h265.mkv"
    [remotion-video-h265-flac.mkv]="https://remotion.media/video-h265-flac.mkv"
    [remotion-video-h265-mp3.mkv]="https://remotion.media/video-h265-mp3.mkv"
    [remotion-video-h265-opus.mkv]="https://remotion.media/video-h265-opus.mkv"
    [remotion-video-h265-vorbis.mkv]="https://remotion.media/video-h265-vorbis.mkv"
    [remotion-video-h265-none.mkv]="https://remotion.media/video-h265-none.mkv"
    [remotion-video-vp8.mkv]="https://remotion.media/video-vp8.mkv"
    [remotion-video-vp8-flac.mkv]="https://remotion.media/video-vp8-flac.mkv"
    [remotion-video-vp8-mp3.mkv]="https://remotion.media/video-vp8-mp3.mkv"
    [remotion-video-vp8-opus.mkv]="https://remotion.media/video-vp8-opus.mkv"
    [remotion-video-vp8-vorbis.mkv]="https://remotion.media/video-vp8-vorbis.mkv"
    [remotion-video-vp8-none.mkv]="https://remotion.media/video-vp8-none.mkv"
    [remotion-video-vp9.mkv]="https://remotion.media/video-vp9.mkv"
    [remotion-video-vp9-flac.mkv]="https://remotion.media/video-vp9-flac.mkv"
    [remotion-video-vp9-mp3.mkv]="https://remotion.media/video-vp9-mp3.mkv"
    [remotion-video-vp9-opus.mkv]="https://remotion.media/video-vp9-opus.mkv"
    [remotion-video-vp9-vorbis.mkv]="https://remotion.media/video-vp9-vorbis.mkv"
    [remotion-video-vp9-none.mkv]="https://remotion.media/video-vp9-none.mkv"
    [remotion-video-prores.mkv]="https://remotion.media/video-prores.mkv"
    [remotion-video-prores-flac.mkv]="https://remotion.media/video-prores-flac.mkv"
    [remotion-video-prores-mp3.mkv]="https://remotion.media/video-prores-mp3.mkv"
    [remotion-video-prores-opus.mkv]="https://remotion.media/video-prores-opus.mkv"
    [remotion-video-prores-vorbis.mkv]="https://remotion.media/video-prores-vorbis.mkv"
    [remotion-video-prores-none.mkv]="https://remotion.media/video-prores-none.mkv"
    [remotion-video.mov]="https://remotion.media/video.mov"
    [remotion-video-pcm.mov]="https://remotion.media/video-pcm.mov"
    [remotion-video-none.mov]="https://remotion.media/video-none.mov"
    [remotion-video-h265.mov]="https://remotion.media/video-h265.mov"
    [remotion-video-h265-pcm.mov]="https://remotion.media/video-h265-pcm.mov"
    [remotion-video-h265-none.mov]="https://remotion.media/video-h265-none.mov"
    [remotion-video-prores.mov]="https://remotion.media/video-prores.mov"
    [remotion-video-prores-pcm.mov]="https://remotion.media/video-prores-pcm.mov"
    [remotion-video-prores-none.mov]="https://remotion.media/video-prores-none.mov"
    [remotion-video-360p.mp4]="https://remotion.media/video-360p.mp4"
    [remotion-video-480p.mp4]="https://remotion.media/video-480p.mp4"
    [remotion-video-720p.mp4]="https://remotion.media/video-720p.mp4"
    [remotion-video-1080p.mp4]="https://remotion.media/video-1080p.mp4"
    [remotion-video-1440p.mp4]="https://remotion.media/video-1440p.mp4"
    [remotion-video-2160p.mp4]="https://remotion.media/video-2160p.mp4"
    [remotion-video-5s.mp4]="https://remotion.media/video-5s.mp4"
    [remotion-video-10s.mp4]="https://remotion.media/video-10s.mp4"
    [remotion-video-30s.mp4]="https://remotion.media/video-30s.mp4"
    [remotion-video-1m.mp4]="https://remotion.media/video-1m.mp4"
    [remotion-video-5m.mp4]="https://remotion.media/video-5m.mp4"
    [remotion-video-10m.mp4]="https://remotion.media/video-10m.mp4"
    [remotion-video-30m.mp4]="https://remotion.media/video-30m.mp4"
    [remotion-video-24fps.mp4]="https://remotion.media/video-24fps.mp4"
    [remotion-video-25fps.mp4]="https://remotion.media/video-25fps.mp4"
    [remotion-video-29.97fps.mp4]="https://remotion.media/video-29.97fps.mp4"
    [remotion-video-30fps.mp4]="https://remotion.media/video-30fps.mp4"
    [remotion-video-59.94fps.mp4]="https://remotion.media/video-59.94fps.mp4"
    [remotion-video-60fps.mp4]="https://remotion.media/video-60fps.mp4"
    [remotion-video-120fps.mp4]="https://remotion.media/video-120fps.mp4"
    [remotion-video-240fps.mp4]="https://remotion.media/video-240fps.mp4"
    [remotion-multiple-audio-streams.mov]="https://remotion.media/multiple-audio-streams.mov"
    [remotion-greenscreen.mp4]="https://remotion.media/greenscreen.mp4"
    [remotion-first-frame-at-4sec.webm]="https://remotion.media/first-frame-at-4sec.webm"
    [remotion-audio-shorter-than-video.mp4]="https://remotion.media/audio-shorter-than-video.mp4"

    [elecard-avs3-sfti-640x360.bin]="https://www.elecard.com/storage/video/SFTI_640x360.bin"
    [elecard-avs3-sfti-854x480.bin]="https://www.elecard.com/storage/video/SFTI_854x480.bin"
    [elecard-avs3-sfti-1280x720.bin]="https://www.elecard.com/storage/video/SFTI_1280x720.bin"
    [elecard-avs3-sfti-1920x1080.bin]="https://www.elecard.com/storage/video/SFTI_1920x1080.bin"
    [elecard-avs3-sfti-3840x2160.bin]="https://www.elecard.com/storage/video/SFTI_3840x2160.bin"
    [elecard-vvc-novosobornaya-square-640x360.mp4]="https://www.elecard.com/storage/video/NovosobornayaSquare_640x360.mp4"
    [elecard-vvc-novosobornaya-square-854x480.mp4]="https://www.elecard.com/storage/video/NovosobornayaSquare_854x480.mp4"
    [elecard-vvc-novosobornaya-square-1280x720.mp4]="https://www.elecard.com/storage/video/NovosobornayaSquare_1280x720.mp4"
    [elecard-vvc-novosobornaya-square-1920x1080.mp4]="https://www.elecard.com/storage/video/NovosobornayaSquare_1920x1080.mp4"
    [elecard-vvc-novosobornaya-square-3840x2160.mp4]="https://www.elecard.com/storage/video/NovosobornayaSquare_3840x2160.mp4"
    [elecard-av1-city-hall-640x360.webm]="https://www.elecard.com/storage/video/CityHall_640x360.webm"
    [elecard-av1-city-hall-854x480.webm]="https://www.elecard.com/storage/video/CityHall_854x480.webm"
    [elecard-av1-city-hall-1280x720.webm]="https://www.elecard.com/storage/video/CityHall_1280x720.webm"
    [elecard-av1-city-hall-1920x1080.webm]="https://www.elecard.com/storage/video/CityHall_1920x1080.webm"
    [elecard-av1-city-hall-3840x2160.webm]="https://www.elecard.com/storage/video/CityHall_3840x2160.webm"
    [elecard-vp9-ushaika-river-640x360.webm]="https://www.elecard.com/storage/video/UshaikaRiverEmb_640x360.webm"
    [elecard-vp9-ushaika-river-854x480.webm]="https://www.elecard.com/storage/video/UshaikaRiverEmb_854x480.webm"
    [elecard-vp9-ushaika-river-1280x720.webm]="https://www.elecard.com/storage/video/UshaikaRiverEmb_1280x720.webm"
    [elecard-vp9-ushaika-river-1920x1080.webm]="https://www.elecard.com/storage/video/UshaikaRiverEmb_1920x1080.webm"
    [elecard-vp9-ushaika-river-3840x2160.webm]="https://www.elecard.com/storage/video/UshaikaRiverEmb_3840x2160.webm"
    [elecard-hevc-tsu-640x360.mp4]="https://www.elecard.com/storage/video/TSU_640x360.mp4"
    [elecard-hevc-tsu-854x480.mp4]="https://www.elecard.com/storage/video/TSU_854x480.mp4"
    [elecard-hevc-tsu-1280x720.mp4]="https://www.elecard.com/storage/video/TSU_1280x720.mp4"
    [elecard-hevc-tsu-1920x1080.mp4]="https://www.elecard.com/storage/video/TSU_1920x1080.mp4"
    [elecard-hevc-tsu-3840x2160.mp4]="https://www.elecard.com/storage/video/TSU_3840x2160.mp4"
    [elecard-h264-theater-square-640x360.mp4]="https://www.elecard.com/storage/video/TheaterSquare_640x360.mp4"
    [elecard-h264-theater-square-854x480.mp4]="https://www.elecard.com/storage/video/TheaterSquare_854x480.mp4"
    [elecard-h264-theater-square-1280x720.mp4]="https://www.elecard.com/storage/video/TheaterSquare_1280x720.mp4"
    [elecard-h264-theater-square-1920x1080.mp4]="https://www.elecard.com/storage/video/TheaterSquare_1920x1080.mp4"
    [elecard-h264-theater-square-3840x2160.mp4]="https://www.elecard.com/storage/video/TheaterSquare_3840x2160.mp4"
    [elecard-mpeg2-wooden-architecture-640x360.ts]="https://www.elecard.com/storage/video/WoodenArchitecture_640x360.ts"
    [elecard-mpeg2-wooden-architecture-854x480.ts]="https://www.elecard.com/storage/video/WoodenArchitecture_854x480.ts"
    [elecard-mpeg2-wooden-architecture-1280x720.ts]="https://www.elecard.com/storage/video/WoodenArchitecture_1280x720.ts"
    [elecard-mpeg2-wooden-architecture-1920x1080.ts]="https://www.elecard.com/storage/video/WoodenArchitecture_1920x1080.ts"

    [test-videos-av1-big-buck-bunny-360-10s-1mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/av1/360/Big_Buck_Bunny_360_10s_1MB.mp4"
    [test-videos-av1-big-buck-bunny-720-10s-2mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/av1/720/Big_Buck_Bunny_720_10s_2MB.mp4"
    [test-videos-av1-big-buck-bunny-1080-10s-5mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/av1/1080/Big_Buck_Bunny_1080_10s_5MB.mp4"
    [test-videos-h264-big-buck-bunny-360-10s-1mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_1MB.mp4"
    [test-videos-h264-big-buck-bunny-720-10s-2mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_2MB.mp4"
    [test-videos-h264-big-buck-bunny-1080-10s-5mb.mp4]="https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/1080/Big_Buck_Bunny_1080_10s_5MB.mp4"
    [test-videos-vp9-big-buck-bunny-360-10s-1mb.webm]="https://test-videos.co.uk/vids/bigbuckbunny/webm/vp9/360/Big_Buck_Bunny_360_10s_1MB.webm"
    [test-videos-vp9-big-buck-bunny-720-10s-2mb.webm]="https://test-videos.co.uk/vids/bigbuckbunny/webm/vp9/720/Big_Buck_Bunny_720_10s_2MB.webm"
    [test-videos-vp9-big-buck-bunny-1080-10s-5mb.webm]="https://test-videos.co.uk/vids/bigbuckbunny/webm/vp9/1080/Big_Buck_Bunny_1080_10s_5MB.webm"
    [test-videos-h264-big-buck-bunny-360-10s-1mb.mkv]="https://test-videos.co.uk/vids/bigbuckbunny/mkv/360/Big_Buck_Bunny_360_10s_1MB.mkv"
    [test-videos-h264-big-buck-bunny-720-10s-2mb.mkv]="https://test-videos.co.uk/vids/bigbuckbunny/mkv/720/Big_Buck_Bunny_720_10s_2MB.mkv"
    [test-videos-h264-big-buck-bunny-1080-10s-5mb.mkv]="https://test-videos.co.uk/vids/bigbuckbunny/mkv/1080/Big_Buck_Bunny_1080_10s_5MB.mkv"

    [d-sav360-videos.zip]="https://zenodo.org/records/19052701/files/360videos_with_ambisonic.zip?download=1"
    [d-sav360-depth.zip]="https://zenodo.org/records/19052701/files/depth.zip?download=1"
)

require_space() {
    local available
    available=$(df -B1 --output=avail "$directory" | tail -n 1)
    if (( available < $1 )); then
        printf 'Need %s bytes free in %s\n' "$1" "$directory" >&2
        exit 1
    fi
}

extract_zip() {
    local archive=$1 destination=${1%.zip} nested
    require_space "$(unzip -l "$archive" | awk 'END {print $1}')"
    unzip -n "$archive" -d "$destination"
    while IFS= read -r -d '' nested; do
        extract_zip "$nested"
    done < <(find "$destination" -type f -name '*.zip' -print0)
}

process_file() {
    local name=$1 file="$directory/$1"
    if [[ -e "$file" ]]; then
        printf 'Skipping %s\n' "$name"
    else
        printf 'Downloading %s\n' "$name"
        if ! curl --silent --show-error --fail --location --retry 3 --continue-at - \
            --output "$file.part" "$2"; then
            printf 'Skipping failed download: %s\n' "$name" >&2
            return
        fi
        mv --no-clobber -- "$file.part" "$file"
    fi
    if [[ "$file" == *.zip ]]; then
        extract_zip "$file"
    fi
}

directory=${1:-$HOME/Videos/VRTestVideos}
jobs=${JOBS:-4}
if ! [[ "$jobs" =~ ^[1-9][0-9]*$ ]]; then
    printf 'JOBS must be a positive integer\n' >&2
    exit 1
fi
mkdir -p -- "$directory"
require_space 30000000000
export directory
export -f process_file extract_zip require_space
for name in "${!downloads[@]}"; do
    printf '%s\0%s\0' "$name" "${downloads[$name]}"
done | xargs -0 -r -n 2 -P "$jobs" bash -euo pipefail -c 'process_file "$@"' _

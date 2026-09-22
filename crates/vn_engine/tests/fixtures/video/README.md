# Video fixtures

These one-second clips contain an FFmpeg-generated 96×64 test pattern at 12 fps
and a 440 Hz sine wave at 48 kHz, stereo. No third-party footage or recordings
are included. H.264 and HEVC files test rejection, not supported desktop playback.

Generate the AV1/Vorbis fixture:

```sh
ffmpeg -f lavfi -i testsrc2=size=96x64:rate=12:duration=1 -f lavfi -i sine=frequency=440:sample_rate=48000:duration=1 -c:v libaom-av1 -cpu-used 8 -crf 45 -pix_fmt yuv420p -threads 1 -c:a libvorbis -ac 2 av1-vorbis.webm
ffmpeg -i av1-vorbis.webm -an -c:v copy silent.webm
```

For `vp9-opus.webm`, use `-c:v libvpx-vp9 -c:a libopus`, omitting the AV1-specific
`-cpu-used 8 -crf 45` arguments. For `h264.mp4`, use `-c:v libx264 -c:a aac`.
For `hevc.mp4`, use `-c:v libx265 -c:a aac` and
`-x265-params pools=none:frame-threads=1:log-level=error`.

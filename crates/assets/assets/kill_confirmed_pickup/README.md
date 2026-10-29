# Kill Confirmed pickup sound

`coin_received.mp3` is the source sound supplied for the Kill Confirmed
prototype. Its redistribution license has not been recorded yet; confirm it
before a public release.

The build embeds two PCM clips, not the MP3: the normal pickup and a one-third
length reversed version for friendly (denied) pickups. Regenerate them with:

```sh
ffmpeg -i coin_received.mp3 -ac 2 -ar 48000 -sample_fmt s16 -f s16le coin_received.pcm
ffmpeg -i coin_received.mp3 -af 'areverse,atrim=duration=0.383125,afade=t=out:st=0.363125:d=0.02' -ac 2 -ar 48000 -sample_fmt s16 -f s16le coin_received_reverse.pcm
```

The PCM is 48 kHz, 16-bit, signed little-endian stereo. The source MP3 SHA-256
is `f70d754b410eb8046ce40721441ff6f6f68605cf31ded7480ee218c7f7356f34`;
the normal and reversed PCM SHA-256 values are
`05438765f3c8e219c16be903f85cfee3498c5f04f602bcb7bbee752d2a0da84c` and
`31929db38037d00176cb93e55a7834507c9f43b21713429bbb8e6dc46da11033`.

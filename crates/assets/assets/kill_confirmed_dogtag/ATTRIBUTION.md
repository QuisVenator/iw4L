# Kill Confirmed dog tag asset

The 3D source model is **Dog tags - Military** by **Nik Ladoukakis**, licensed
under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

- Source: https://sketchfab.com/3d-models/dog-tags-military-9879d4ab79944b4fb927cfa7ae827808
- Author: https://sketchfab.com/nikladoukakis
- Vendored source: `dog_tags_-_military.glb`
- Source SHA-256: `c1548999dc84e34f87b7c5ebddf7674ba9ed7840af19cb2df1506c4cd4b7133d`

`mesh.bin`, `friendly.png`, and `enemy.png` are generated derivatives for
iw4L. The blue/orange tints distinguish friendly and enemy Kill Confirmed tags.

Regenerate them with:

```sh
python3 crates/assets/tools/prepare_kill_confirmed_dogtag.py \
  crates/assets/assets/kill_confirmed_dogtag/dog_tags_-_military.glb \
  crates/assets/assets/kill_confirmed_dogtag
```

The preprocessing tool needs Python 3, NumPy, and Pillow. It is not part of the
Rust build.

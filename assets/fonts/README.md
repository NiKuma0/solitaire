# Fonts

Inter 4.1 by The Inter Project Authors, SIL Open Font License 1.1 (see `OFL.txt`).
Source: https://github.com/rsms/inter/releases/tag/v4.1

The files here are subsets (Basic Latin plus a few punctuation marks) of
`extras/ttf/Inter-Regular.ttf` and `extras/ttf/Inter-SemiBold.ttf`, made with:

```sh
uvx --from fonttools pyftsubset Inter-Regular.ttf \
  --unicodes="U+0020-007E,U+00A0,U+00B7,U+00D7,U+2013,U+2014,U+2026" \
  --layout-features="kern" --no-hinting --desubroutinize \
  --output-file=Inter-Regular.ttf
```

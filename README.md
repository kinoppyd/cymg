# cymg

`cymg` is a CLI tool that applies a cyberpunk-style image effect.

## Usage

```bash
cymg [options] <image-path>
```

- Input: one image file path
- Output: saved in the same directory as:
  - `<original_stem>.cyber.<original_ext>`
  - with `--random`: `<original_stem>.cyber.<index>.<original_ext>` (`index` starts at `0`)
  - with `--animation`: `<original_stem>.cyber.gif`

Example:

- Input: `photo.jpg`
- Output: `photo.cyber.jpg`

Show help:

```bash
cymg --help
```

## Sample Output

Reference image:

- https://avatars.githubusercontent.com/u/2846039?v=4

Generated files:

- Source: `docs/images/sample-source.png`
- Processed: `docs/images/sample-cyber.png`

Command used:

```bash
cymg --green --scanline 0.72 --scanline-step 2 --glitch-rate 0.24 --bloom 0.36 --noise 0.14 --contrast 1.22 docs/images/sample-source.png
```

| Source | Cyberpunk Output |
| --- | --- |
| ![Source sample](docs/images/sample-source.png) | ![Cyber sample](docs/images/sample-cyber.png) |

## Option Reference

### Color Mode (Mutually Exclusive)

`--red`, `--green`, `--blue`

- Selects the dominant color direction.
- You can specify only one of these flags at a time.
- `--green` and `--blue` are intentionally strong and include highlight recoloring behavior.

### Global Effect Strength

`--intensity <0.0-2.0>` (default: `1.0`)

- Overall blend amount of processed result against the source.
- `0.0`: mostly close to the source image.
- `1.0`: standard full effect.
- `>1.0`: exaggerated result (can clip highlights and shadows faster).

### Glitch Controls

`--glitch-shift <0-128>` (default: `9`)

- Max RGB channel offset in pixels.
- Larger values increase horizontal color separation and glitch displacement.

`--glitch-rate <0.0-1.0>` (default: `0.18`)

- Frequency of stronger glitch rows.
- Higher values create more frequent line-level disruption.

### Scanline Controls

`--scanline <0.0-1.0>` (default: `0.62`)

- Scanline strength.
- Higher values create stronger dark/bright line contrast.

`--scanline-step <1-32>` (default: `2`)

- Vertical interval between scanline phases.
- Lower values create denser lines.
- Higher values create wider spacing.

### Tone and Contrast

`--vignette <0.0-1.0>` (default: `0.45`)

- Edge darkening strength.
- Higher values pull attention toward the image center.

`--brightness <-1.0-1.0>` (default: `0.0`)

- Linear brightness shift.
- Positive values lift overall luminance; negative values darken.

`--contrast <0.0-3.0>` (default: `1.08`)

- Contrast gain around mid-gray.
- `1.0` is near neutral.
- `>1.0` increases separation between dark and bright regions.

`--saturation <0.0-3.0>` (default: `1.15`)

- Color vividness multiplier.
- `0.0` approaches monochrome.
- `>1.0` increases chroma intensity.

### Texture and Glow

`--noise <0.0-1.0>` (default: `0.08`)

- Random film/grain intensity.
- Higher values increase gritty texture and roughness.

`--bloom <0.0-1.0>` (default: `0.20`)

- Neon glow amount around bright areas.
- Higher values create wider and stronger light bleed.

### Determinism

`--seed <u64>` (default: `2077`)

- Seed for pseudo-random parts (glitch/noise).
- Same input + same options + same seed gives repeatable results.
- Change seed to get another variation.

### Random Batch

`--random`

- Generates 10 images in one run with randomized effect parameters.
- Output names are numbered from `0` to `9`.
- The selected color mode (`--red` / `--green` / `--blue`) is preserved and is not randomized.
- If no color mode is specified, color mode remains neutral for all 10 outputs.

### Animation GIF

`--animation`

- Internally runs random 10-image generation first.
- Builds one GIF from those randomized outputs.
- Output file name is `<original_stem>.cyber.gif`.
- Does not write numbered `<original_stem>.cyber.<index>.<ext>` files.
- For performance, very large inputs are automatically resized for GIF encoding within `800x600` while preserving aspect ratio.
- Frame behavior:
  - frame `0` is the first keyframe and is shown longer first.
  - frames `1..3` are shown briefly one by one, returning to frame `0`.
  - frame `4` is the second keyframe and is also shown for the same long duration.
  - frames `5..9` are shown briefly; between them it returns to frame `4` (except after frame `9`).
  - after frame `9`, loop restart returns directly to frame `0`.
- The GIF loops infinitely.

### Help

`-h`, `--help`

- Prints usage and examples.

## Examples

Basic:

```bash
cymg input.jpg
```

Strong red glitch:

```bash
cymg --red --intensity 1.35 --glitch-shift 14 --glitch-rate 0.28 input.jpg
```

Visible scanlines + blue tone:

```bash
cymg --blue --scanline 0.70 --scanline-step 2 --vignette 0.60 input.png
```

High-contrast green cinematic look:

```bash
cymg --green --contrast 1.30 --saturation 1.35 --noise 0.20 --bloom 0.35 --seed 42 input.webp
```

Randomized 10-image batch while keeping blue tone:

```bash
cymg --blue --random input.jpg
```

Animation mode (runs random batch internally):

```bash
cymg --green --animation input.jpg
```

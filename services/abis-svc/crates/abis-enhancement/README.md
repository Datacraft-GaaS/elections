# eel-abis-enhancement

Biometric image enhancement pipeline for fingerprint restoration, face super-resolution, slap segmentation, and iris segmentation.

## Fingerprint Restoration

`GrayscaleImage` is the core image type: a width, height, and pixel buffer. `new()` validates buffer length.

### Histogram Equalization

`histogram_equalize` flattens the intensity histogram using the CDF-based standard formula: `new_val = round(255 * (cdf[p] - cdf_min) / (total - cdf_min))`. Improves contrast on low-quality captures.

### Gabor Filter Bank

`gabor_filter` convolves an image with a 15x15 Gabor kernel at a given frequency and orientation. The kernel combines a Gaussian envelope with a cosine wave aligned to the orientation. `apply_gabor_bank` applies 8 orientations (0 to 7pi/8) and averages the results, enhancing ridge structure across all angles.

### Morphological Thinning

`morphological_thin` binarizes the image at a threshold then applies two rounds of 3x3 erosion. This is a simplified thinning stub. Hilditch or Zhang-Suen thinning should replace this for production use where skeleton quality matters for minutiae extraction.

### Restoration Pipeline

`restore` chains: histogram equalization, Gabor bank filtering, morphological thinning at threshold 128.

## Face Super-Resolution

`bicubic_upsample` scales a grayscale image by an integer factor using bilinear interpolation (a bicubic-named stub). `unsharp_mask` sharpens via box blur and additive residual: `sharp = orig + amount * (orig - blurred)`. `super_resolve` chains both steps. Production wires a super-resolution neural model (e.g. ESRGAN) via ONNX Runtime.

## Slap Segmentation

`segment_slap` divides a 4-finger slap image into equal-width strips and assigns finger positions. Right hand: Index, Middle, Ring, Little from left to right. Left hand: Little, Ring, Middle, Index from left to right. Returns an array of 4 `SegmentedFinger` records with bounding boxes and cropped images. Fails if slap width is less than 4 pixels.

## Iris Segmentation

`integro_differential_operator` implements a Daugman-style integro-differential search: it evaluates candidate (center, radius) triples on a coarse grid (step=5), measuring contrast between inner and outer annular averages at each candidate. The triple with maximum contrast is returned as the best circle.

`detect_eyelid_occlusion` marks the top 15% and bottom 15% of the limbus bounding box as occluded, approximating eyelid coverage.

`segment_iris` chains pupil detection (r=10..80), limbus detection (r=80..img_min/2), and eyelid occlusion into a single `IrisSegmentation`.

## Error Handling

All fallible functions return `Result<_, EnhancementError>`. No `unwrap()` in library code. Errors:

- `InvalidDimensions { width, height }`: image too small or buffer size mismatch
- `InsufficientData(String)`: not enough data to operate
- `InvalidParameter(String)`: bad configuration value
- `SegmentationFailed(String)`: segmentation could not complete

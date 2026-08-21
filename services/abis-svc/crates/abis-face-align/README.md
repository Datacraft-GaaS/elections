# eel-abis-face-align

Face alignment for the EEL ABIS pipeline.

Computes the Umeyama similarity transform (rotation + scale + translation)
mapping 5 detected landmarks to the ArcFace 112x112 canonical template.
Warps the source image via bilinear interpolation.

## References

- Deng et al. (2019). ArcFace: Additive Angular Margin Loss for Deep Face
  Recognition. CVPR 2019.
- Umeyama, S. (1991). Least-squares estimation of transformation parameters
  between two point patterns. IEEE TPAMI 13(4):376-380.
- Sagonas et al. (2013). 300 Faces In-The-Wild Challenge. ICCV 2013.

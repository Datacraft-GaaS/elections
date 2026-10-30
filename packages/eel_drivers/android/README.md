# Android native driver inputs

The Android build of `eel_drivers` links against a closed third-party sensor SDK
(AntIris USB camera and iris stack). Those artefacts are not redistributable, so
they are not in this repository and the `libs/` and `src/main/jniLibs/`
directories are empty on a fresh clone.

Without them, `build.gradle` still resolves (`fileTree` over an empty `libs/`
is a no-op), but `Biometrics.kt` and the AntIris plugin will not compile.

## Provisioning

Obtain the SDK from the vendor under your own licence and place it here:

```
android/libs/
  antiris.antusbcamera.jar
  serenegiant.jar
android/src/main/jniLibs/
  arm64-v8a/{lib_Antiris,libUSBCamera,libopenjpeg,libpng}.so
  armeabi-v7a/{lib_Antiris,libUSBCamera,libopenjpeg,libpng}.so
```

Verify the SHA-256 of each file against the vendor manifest before use. `*.jar`,
`*.aar` and `*.so` are ignored by git deliberately: a vendor blob must never be
committed, and the ignore rules are what enforce that.

Builds that do not need a physical iris sensor (contract tests, simulation,
`kit-sdk` transports) run without these files.

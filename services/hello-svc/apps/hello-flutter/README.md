# hello-flutter

Hello-world Flutter app used as scaffold template and CI smoke-test for kit-sdk-flutter.

## Purpose

Scaffolding template and CI smoke-test for the `packages/kit-sdk-flutter` shared package.
Not a production application. Demonstrates the minimum viable Flutter app structure,
MDM config injection, and CI pipeline integration.

## Build Commands

```bash
cd services/hello-svc/apps/hello-flutter
flutter build apk --release
flutter build ios --release   # macOS only
flutter build web
```

## Test Commands

```bash
flutter test
```

## Notes

- Do not add this app to production MDM profiles
- Do not submit to public app stores
- Used only for CI pipeline validation and as a template for new kit apps

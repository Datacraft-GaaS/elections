# eel Dart API Client

Generated Dart/Dio clients for all eel OpenAPI contexts.

## Regenerate

From the repo root:

```sh
just gen-clients
```

Or from this directory:

```sh
./generate.sh --regen
```

## What the generated code contains

For each OpenAPI context (e.g. `elections`, `results`), a Dart package is generated under `generated/<ctx>/` using the `dart-dio` generator. Each package includes:

- Typed model classes (Pydantic-style, but in Dart)
- A `DefaultApi` class using the `dio` HTTP client
- Serialization helpers via `json_serializable`

## Usage

Add the generated package to your `pubspec.yaml`:

```yaml
dependencies:
  eel_api_client_elections:
    path: ../api-clients/dart/generated/elections
```

Then in Dart:

```dart
import 'package:eel_api_client_elections/eel_api_client_elections.dart';

final api = EelApiClientElections().getDefaultApi();
final response = await api.listElections();
```

## Notes

- The `generated/` directory is gitignored. Run `./generate.sh --regen` after pulling new OpenAPI specs.
- Requires `openapi-generator-cli` (via npm, Homebrew, or Docker) and optionally `dart` for formatting.
- Config: `openapi-generator-config.yaml`

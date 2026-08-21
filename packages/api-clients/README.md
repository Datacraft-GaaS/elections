# api-clients

Auto-generated API clients for all eel OpenAPI contracts in four languages. Do not hand-edit these files.

## Purpose in context

Client code for consuming eel REST APIs is generated from `packages/openapi-contracts/` by `just gen-clients`. Generated clients are committed to the repository so that downstream consumers do not need to run the generator themselves. Any hand-edit will be overwritten on the next `just gen-clients` run.

## Layout

```
api-clients/
  dart/       # For Flutter consumers (kit-sdk-flutter, mobile apps)
  py/         # For internal Python tooling and test harnesses
  rs/         # For internal service-to-service calls in Rust
  ts/         # For Astro/Svelte portal frontends
  verify.sh   # Checks that generated files are up to date with specs
```

## Regenerating

```bash
# Regenerate all clients:
just gen-clients

# Regenerate a single language:
just gen-clients --lang dart
just gen-clients --lang ts

# Regenerate a single bounded context:
just gen-clients --context rts

# Verify generated clients match current specs (CI check):
bash packages/api-clients/verify.sh
```

## Consuming (Dart)

```dart
// pubspec.yaml
dependencies:
  eel_api_clients:
    path: ../../packages/api-clients/dart
```

```dart
import 'package:eel_api_clients/rts.dart';

final client = RtsApiClient(baseUrl: 'https://rts.example.local');
final result = await client.submitForm34a(Form34ARequest(stationId: '47-001', ...));
```

## Consuming (TypeScript)

```typescript
import { RtsApiClient } from '@eel/api-clients/rts';

const client = new RtsApiClient({ baseUrl: 'https://rts.example.local' });
const result = await client.getResults({ electionId: '2027-general' });
```

## Consuming (Python)

```python
from eel_api_clients.rts import RtsApiClient

client = RtsApiClient(base_url="https://rts.example.local")
results = await client.get_results(election_id="2027-general")
```

## CI check

CI runs `bash packages/api-clients/verify.sh` on every PR that touches `packages/openapi-contracts/`. The check regenerates clients in a temp directory and diffs against the committed files. A non-empty diff fails the build, requiring the PR author to commit updated clients.

## Related docs

- `packages/openapi-contracts/README.md` - source OpenAPI specs
- `docs/adr/0011-api-conventions.md` - API conventions enforced in generated clients

## Limitations

Generated clients do not include retry logic, authentication, or load shedding. Consumers must add these using the appropriate `eel-*` package for their language. The Rust client (`rs/`) uses `reqwest`; the Dart client uses `dio`; the TypeScript client uses `fetch`.

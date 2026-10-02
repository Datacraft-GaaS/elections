/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * eel-has-data-class
 *
 * Spectral custom function: checks that every property in a schema has an
 * x-eel-data-class annotation (A, B, C, D, or E) per ADR-0016.
 *
 * Usage in ruleset:
 *   then:
 *     function: eel-has-data-class
 */
const VALID_CLASSES = new Set(["A", "B", "C", "D", "E"]);

function checkProperties(schema, path, results) {
  if (!schema || typeof schema !== "object") return;

  const props = schema.properties;
  if (!props) return;

  for (const [propName, propSchema] of Object.entries(props)) {
    const propPath = [...path, "properties", propName];
    const dc = propSchema["x-eel-data-class"];
    if (dc === undefined || dc === null) {
      results.push({
        message: `Property '${propName}' is missing x-eel-data-class annotation (must be A, B, C, D, or E per ADR-0016)`,
        path: propPath,
      });
    } else if (!VALID_CLASSES.has(String(dc).toUpperCase())) {
      results.push({
        message: `Property '${propName}' has invalid x-eel-data-class '${dc}' (must be A, B, C, D, or E)`,
        path: propPath,
      });
    }
    // Recurse into nested objects
    if (propSchema.properties) {
      checkProperties(propSchema, propPath, results);
    }
    if (propSchema.items && propSchema.items.properties) {
      checkProperties(propSchema.items, [...propPath, "items"], results);
    }
  }
}

export default function eelHasDataClass(schema, _opts, context) {
  const results = [];
  checkProperties(schema, context.path ?? [], results);
  return results;
}

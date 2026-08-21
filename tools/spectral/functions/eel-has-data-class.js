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

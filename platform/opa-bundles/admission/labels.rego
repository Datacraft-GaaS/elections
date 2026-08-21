package eel.admission.labels

import future.keywords.if
import future.keywords.in

# Validates that Kubernetes resources carry the required eel.internal/* labels.

required_labels := {
	"eel.internal/managed-by",
	"eel.internal/service",
}

deny[msg] {
	input.request.kind.kind in {"Deployment", "StatefulSet", "DaemonSet"}
	label := required_labels[_]
	not input.request.object.metadata.labels[label]
	msg := sprintf("Resource %s/%s is missing required label %s", [
		input.request.object.metadata.namespace,
		input.request.object.metadata.name,
		label,
	])
}

deny[msg] {
	input.request.kind.kind == "Namespace"
	not input.request.object.metadata.labels["eel.internal/managed-by"]
	msg := sprintf("Namespace %s must have label eel.internal/managed-by", [input.request.object.metadata.name])
}

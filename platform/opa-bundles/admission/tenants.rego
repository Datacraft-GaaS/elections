package eel.admission.tenants

import future.keywords.if
import future.keywords.in

# Tenant isolation: resources labelled with eel.internal/tenant must not
# cross-reference resources from a different tenant.

deny[msg] if {
	input.request.kind.kind in {"ConfigMap", "Secret"}
	tenant := input.request.object.metadata.labels["eel.internal/tenant"]
	tenant != ""
	ns_tenant := data.tenants[input.request.object.metadata.namespace]
	ns_tenant != tenant
	msg := sprintf("Resource %s/%s has tenant %s but namespace is owned by tenant %s", [
		input.request.object.metadata.namespace,
		input.request.object.metadata.name,
		tenant,
		ns_tenant,
	])
}

# Election-day freeze: deny mutations to T1 namespaces during freeze window
deny[msg] if {
	data.election.freeze_active == true
	input.request.operation in {"CREATE", "UPDATE", "DELETE"}
	input.request.object.metadata.namespace in data.election.t1_namespaces
	input.request.userInfo.username != "system:serviceaccount:argocd:argocd-application-controller"
	msg := sprintf("Election freeze is active: mutations to namespace %s are blocked", [input.request.object.metadata.namespace])
}

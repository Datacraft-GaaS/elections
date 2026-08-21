// Sonatype Nexus OSS bootstrap script.
// Run once via Nexus scripting API:
//   curl -u admin:$NEXUS_PASS -H "Content-Type: application/json" \
//     http://nexus.internal:8081/service/rest/v1/script \
//     -d @nexus-bootstrap.json   (wraps this file as the "content" field)
// Then: POST .../run
//
// Naming: eel/nexus-* (no iebc_ prefix)

import org.sonatype.nexus.repository.config.Configuration
import groovy.json.JsonOutput

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

def createProxy(String name, String format, String remoteUrl, Map extra = [:]) {
    def exists = repository.repositoryManager.get(name) != null
    if (exists) {
        log.info("repo $name already exists, skip")
        return
    }
    def cfg = repository.repositoryManager.newConfiguration()
    cfg.repositoryName = name
    cfg.recipeName = "${format}-proxy"
    cfg.online = true
    cfg.attributes = [
        proxy: [
            remoteUrl        : remoteUrl,
            contentMaxAge    : 1440,
            metadataMaxAge   : 1440,
        ],
        httpclient: [
            blocked    : false,
            autoBlock  : true,
            connection : [useTrustStore: true],
        ],
        storage: [
            blobStoreName   : "default",
            strictContentTypeValidation: true,
        ],
        negativeCache: [
            enabled        : true,
            timeToLive     : 1440,
        ],
    ] + extra
    repository.repositoryManager.create(cfg)
    log.info("created proxy repo: $name")
}

def createHosted(String name, String format, Map extra = [:]) {
    def exists = repository.repositoryManager.get(name) != null
    if (exists) { log.info("repo $name already exists, skip"); return }
    def cfg = repository.repositoryManager.newConfiguration()
    cfg.repositoryName = name
    cfg.recipeName = "${format}-hosted"
    cfg.online = true
    cfg.attributes = [
        storage: [
            blobStoreName              : "default",
            strictContentTypeValidation: true,
            writePolicy                : "ALLOW",
        ],
        cleanup: [policyName: ["eel-retention-policy"]],
    ] + extra
    repository.repositoryManager.create(cfg)
    log.info("created hosted repo: $name")
}

// ---------------------------------------------------------------------------
// Proxy repos (upstream mirrors, all traffic goes through these in air-gapped)
// ---------------------------------------------------------------------------

createProxy("eel-pypi-proxy",   "pypi",  "https://pypi.org/")
createProxy("eel-npm-proxy",    "npm",   "https://registry.npmjs.org/")
createProxy("eel-cargo-proxy",  "raw",   "https://static.crates.io/",
    [raw: [contentDisposition: "ATTACHMENT"]])
createProxy("eel-maven-proxy",  "maven2","https://repo1.maven.org/maven2/",
    [maven: [versionPolicy: "MIXED", layoutPolicy: "STRICT"]])
createProxy("eel-apt-proxy",    "apt",   "http://archive.ubuntu.com/ubuntu/",
    [apt: [distribution: "jammy"]])
createProxy("eel-oci-proxy",    "docker","https://registry-1.docker.io/",
    [docker: [httpPort: 5000, httpsPort: null, v1Enabled: false],
     dockerProxy: [indexType: "HUB"]])
createProxy("eel-hex-proxy",    "raw",   "https://repo.hex.pm/",
    [raw: [contentDisposition: "ATTACHMENT"]])

// ---------------------------------------------------------------------------
// Hosted repos (internal releases)
// ---------------------------------------------------------------------------

createHosted("eel-releases",   "raw")
createHosted("eel-snapshots",  "raw")
createHosted("eel-harbor-oci", "docker",
    [docker: [httpPort: 5001, httpsPort: null, v1Enabled: false]])

// ---------------------------------------------------------------------------
// Retention / cleanup policy
// ---------------------------------------------------------------------------

def policyName = "eel-retention-policy"
def existingPolicy = cleanupPolicyStorage.get(policyName)
if (existingPolicy == null) {
    def policy = cleanupPolicyStorage.newCleanupPolicy()
    policy.setName(policyName)
    policy.setMode("DELETION")
    policy.setFormat("ALL_FORMATS")
    policy.setNotes("eel: delete assets not downloaded in 90 days and older than 180 days")
    policy.setCriteria([
        "lastDownloaded": "7776000",   // 90 days
        "lastBlobUpdated": "15552000", // 180 days
    ])
    cleanupPolicyStorage.add(policy)
    log.info("created cleanup policy: $policyName")
} else {
    log.info("cleanup policy $policyName already exists, skip")
}

// ---------------------------------------------------------------------------
// Service account for CI runners
// ---------------------------------------------------------------------------

def userName = "eel-ci-runner"
def securitySystem = container.lookup(org.sonatype.nexus.security.SecuritySystem.class)
try {
    securitySystem.getUser(userName, "default")
    log.info("user $userName already exists, skip")
} catch (Exception ignored) {
    def user = securitySystem.getAuthorizationManager("default")
        .listRoles().find { it.roleId == "nx-anonymous" }
    def newUser = new org.sonatype.nexus.security.user.User()
    newUser.userId = userName
    newUser.firstName = "EEL"
    newUser.lastName = "CI Runner"
    newUser.emailAddress = "ci@eel.internal"
    newUser.source = "default"
    newUser.status = org.sonatype.nexus.security.user.UserStatus.active
    newUser.roles = [new org.sonatype.nexus.security.authz.UserRoleMappingImpl(
        userId: userName, source: "default", roles: ["nx-deployment"]
    )] as Set
    // Password set via env or Vault at deploy time; placeholder triggers a reset.
    securitySystem.addUser(newUser, "changeme-set-via-vault")
    log.info("created user: $userName")
}

log.info("Nexus bootstrap complete")
return JsonOutput.toJson([status: "ok", repos: [
    "eel-pypi-proxy", "eel-npm-proxy", "eel-cargo-proxy", "eel-maven-proxy",
    "eel-apt-proxy", "eel-oci-proxy", "eel-hex-proxy",
    "eel-releases", "eel-snapshots", "eel-harbor-oci"
]])

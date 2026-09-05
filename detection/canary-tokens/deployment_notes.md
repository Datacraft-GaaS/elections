# Canary token deployment — M5 security audit (48h drill)
#
# 1. Build-time injection (build.rs): insert canary files into APK/assets
# 2. Runtime verification (falco + tetragon): detect access attempts
# 3. Audit correlation: canary event -> audit entry (auditId + correlationId)
# 4. Alert routing: Falco ALERT -> audit shadow -> central detection stack
#
# References:
# - docs/mobile/M5_SMS_USB_MANIFEST.md
# - docs/mobile/M3_HARDWARE_INTEGRATION.md

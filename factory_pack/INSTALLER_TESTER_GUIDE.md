# Factory Pack — Installer & Tester Guide
# 工厂包 — 安装与测试指南

> Read this file first. English first, 中文在后.
> 先读本文件。英文在前，中文在后。

---

# Part 1 — English

## 1. Who this guide is for

- **Installers**: put the right APKs on the right devices.
- **Testers**: run the automated checks and judge PASS / FAIL correctly
  (including which FAILs are environmental and do NOT block sign-off).

You do **not** need Flutter, source code, or a network connection to the
build servers. Everything needed is in this directory.

## 2. What is in this directory

| File | Size (approx) | Purpose |
|---|---|---|
| `bvi-kit-debug.apk` | ~180 MB | BVI Kit, debug build — for all drill/test flows |
| `bvr-kit-debug.apk` | ~180 MB | BVR Kit, debug build |
| `rts-kit-debug.apk` | ~160 MB | RTS Kit, debug build |
| `bvi-kit-release.apk` | ~51 MB | BVI Kit, release build (release-path validation only) |
| `bvr-kit-release.apk` | ~51 MB | BVR Kit, release build |
| `rts-kit-release.apk` | ~61 MB | RTS Kit, release build (2% over budget, documented) |
| `factory_test_harness.apk` | ~170 MB | Hardware test app (iris, fingerprint, printer, SMS, scanner, camera, GNSS, NTS, battery, USB, Bluetooth, telephony) |
| `factory_test.sh` | | Master script: runs all checks in order, writes a JSON report |
| `smoke.sh` | | Cold-launch + alive + crash-free check |
| `kill-drill.sh` | | Force-stop at each startup stage, assert recovery |
| `offline-drill.sh` | | Airplane-mode on/off, assert app survives |
| `test_harness.sh` | | Runs the hardware test app, pulls its JSON report |
| `package_test_harness.sh` | | Rebuilds `factory_test_harness_package.zip` |
| `lib/common.sh`, `lib/device.sh` | | Helper libraries (must stay beside the scripts) |
| `antiris/` | | Iris-scanner vendor libraries (device-side dependency) |
| `test_harness/` | | Test-app source (optional reference) |
| `README.md` | | Full pack documentation |
| `factory_guidance.md` | | What to verify about an APK before release |
| `compile_apks.md` | | How the APKs were built + traceability (git commit, SHA-256) |
| `SHA256SUMS` | | Checksums for every file above |
| `INSTALLER_TESTER_GUIDE.md` | | This file |

**Debug vs release — which APK to install?**

- For all automated tests in this guide: **debug** APKs. The test harness
  report can only be read from a debuggable build.
- Release APKs prove the optimized (AOT) build launches and runs. They are
  signed with the **debug key** — they are NOT production builds and must
  NEVER go to MDM/production. See `compile_apks.md`.

## 3. Workstation prerequisites

1. **Android SDK Platform Tools** (`adb`). Check: `adb --version`.
2. A USB cable + a device with **USB debugging enabled**
   (Settings → Developer options → USB debugging). Accept the
   "Allow USB debugging?" prompt on the device — otherwise the device shows
   as `unauthorized` and every script fails.
3. `bash`, `curl`, `shasum` (or `sha256sum`), `unzip` — standard on
   macOS/Linux.
4. No Flutter needed unless you rebuild from source.

Verify the device is visible:

```bash
adb devices
# emulator-5554	device     <- serial + "device" means ready
```

## 4. Verify package integrity (do this first on every new workstation)

```bash
cd factory_pack
shasum -c SHA256SUMS
# every line must end with OK; any FAILED line = re-copy the pack, do not test
```

## 5. Install an app

```bash
# Debug build (for testing). -g pre-grants permissions so no system dialog
# blocks the automated runs (rts-kit asks for CAMERA + MIC at startup).
adb -s <serial> install -r -g bvi-kit-debug.apk
```

Replace `<serial>` with your device serial from `adb devices`, and the APK
name with the app under test. `Success` means installed.

## 6. Run the full automated suite (recommended)

```bash
./factory_test.sh bvi-kit --device <serial>
```

This installs the matching `-debug.apk` (when present beside the script),
then runs, in order: **smoke → kill-drill → offline-drill → test_harness**.
It stops at the first failure and writes
`factory-report-<app>-<serial>.json`, e.g.:

```json
{"app":"bvi-kit","device":"emulator-5554",
 "results":{"smoke.sh":"PASS","kill-drill.sh":"PASS",
            "offline-drill.sh":"PASS","test_harness.sh":"FAIL"},
 "result":"FAIL"}
```

Expected duration: ~6–8 minutes (mostly the 2-minute hardware-app wait).

You can also run stages individually:

```bash
./smoke.sh bvi-kit --device <serial>            # ~40 s
./kill-drill.sh bvi-kit --device <serial>       # ~3 min, 7 stages
./offline-drill.sh bvi-kit --device <serial>    # ~30 s
./test_harness.sh ignore --device <serial>      # ~3 min
```

Optional environment variables (export before running):

| Variable | Effect |
|---|---|
| `EEL_TELEMETRY_URL` | Smoke test probes this URL for a session (e.g. `http://localhost:9090/healthz` with `adb reverse tcp:9090 tcp:9090`). Unset → telemetry reported SKIP (never faked). |
| `EEL_DEDUP_CHECK` | Shell command run after reconnect to verify exactly-once delivery (e.g. query the backend audit log). Must exit 0. Unset → dedup reported SKIP (never a silent pass). |

## 7. How to read results (important — not every FAIL blocks sign-off)

Each check prints a JSON line with `"result":"PASS"` or `"FAIL"` plus a
`reason`. Status meanings:

| Status | Meaning | Blocks sign-off? |
|---|---|---|
| PASS | Check genuinely passed | No |
| FAIL | Check genuinely failed — see `reason` | **Yes**, unless listed below as environmental |
| SKIP | Prerequisite missing (telemetry URL / dedup hook / hardware plugin). Reported explicitly, never assumed. | No, but record it |
| NOT-RUN | A previous stage failed so this one never ran | Fix the earlier failure first |

**Environmental results you should EXPECT (do not block on these):**

| Where | Result | Why | What to do |
|---|---|---|---|
| Emulator, test harness | GNSS → FAIL | No satellite fix indoors/emulator (30 s timeout, by design) | PASS on real hardware with sky view |
| Anywhere without NTS server | NTS → ERROR | `time.eel.local` unreachable in this environment | PASS where the NTS endpoint exists |
| No such hardware attached | Iris, Fingerprint, Printer, Scanner → SKIP (`No ... hardware attached`) | Plugin bundled since 2026-09-18; SKIP now means no peripheral, not missing code | PASS on hardware with the peripheral attached (iris needs Antiris UVC camera; fingerprint/scanner need vendor SDKs — see ENGINEERING_FAQ) |
| Emulator virtual camera / radio | Camera, SMS → PASS | Real Camera2 capture / SmsManager send through emulator virtual devices | Confirms driver works; re-verify on hardware |
| No `EEL_TELEMETRY_URL` | telemetry → SKIP | Nothing configured to probe | Configure it for backend-connected runs |
| No `EEL_DEDUP_CHECK` | dedup → SKIP | No backend hook provided | Provide the hook for sign-off runs |
| Emulator cold start > 2000 ms | WARN only | The 2 s budget targets physical mid-tier phones, not emulators | Measure on reference hardware |

**Results that ALWAYS block sign-off:** any `no-process`, `crash`,
`offline-crash`, `reconnect-death` FAIL; any kill-drill stage FAIL;
a missing `test_report.json` when the harness ran on capable hardware.

## 8. Reading the hardware test report manually

```bash
adb -s <serial> install -r -g factory_test_harness.apk
adb -s <serial> shell monkey -p com.eel.test_harness -c android.intent.category.LAUNCHER 1
# wait ~2 minutes for the 9 hardware tests (GNSS alone can take 30 s)
adb -s <serial> shell run-as com.eel.test_harness cat app_flutter/test_report.json > test_report.json
```

Each entry has `feature`, `status` (PASS/FAIL/SKIP/ERROR), `details`,
`durationMs`. Note the path is `app_flutter/`, not `files/`.

## 9. Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `device ... not online` / empty `adb devices` | Cable, USB debugging off, or unauthorized | Re-plug, enable USB debugging, accept the on-device prompt |
| `no installed package matches 'xyz'` | Typo in app name, or APK not installed | Check name (`bvi-kit`, `bvr-kit`, `rts-kit`); install first |
| `ambiguous fragment` | Two packages match | Uninstall the stale one (`adb uninstall <package>`) |
| A system permission dialog sits on screen and runs stall | App's first-launch permission request (rts-kit: CAMERA + MIC) | Reinstall with `-g`; or grant via `adb shell pm grant <pkg> <permission>` |
| Script exits silently with code 1 and no FAIL line | Old script version | Re-copy the pack (fixed in the 2026-09-18 scripts); never edit scripts by hand |
| `No test report retrieved` | Suite still running (wait longer), or release harness installed | Wait 2 min and retry the pull; the harness APK must be the debug build |
| `run-as ... Permission denied` | Non-debuggable (release) harness installed | Install the debug `factory_test_harness.apk` |
| `INSTALL_FAILED_NO_MATCHING_ABI` | Wrong-ABI APK (only if using per-ABI splits) | Use the universal `-debug.apk` from this pack |
| Cold start WARN over 2000 ms on emulator | Emulator is slow; budget is for hardware | Re-measure on a reference phone |
| `shasum -c` shows FAILED | Corrupt/incomplete copy | Re-copy the pack; do not test |

## 10. Budgets and hard rules

- Cold start → first frame: **≤ 2000 ms on physical mid-tier hardware**.
- APK size: ≤ 40 MB per app, **≤ 60 MB for kits** (release builds).
- Debug APKs (~150–180 MB) are exempt — the size is the debug JIT +
  emulator ABI + validation layers, all absent from release builds.
- Release APKs here are **debug-key signed**: validate the release path
  only. Never stage them to MDM/production.
- Never edit the scripts by hand; never skip a FAIL without a recorded,
  environmental reason from §7.

## 11. Rebuilding the distribution zip (maintainers)

```bash
./package_test_harness.sh --zip --include-src
shasum -c SHA256SUMS
```

This rebuilds the debug harness APK, regenerates `SHA256SUMS`, and writes
`factory_test_harness_package.zip` (slimmed source: Android + shared files
only, no desktop/iOS dirs, no IDE files).

---

# Part 2 — 中文 (简体中文)

## 1. 本指南的读者

- **安装人员**：把正确的 APK 装到正确的设备上。
- **测试人员**：运行自动化检查并正确判定 PASS / FAIL（包括哪些 FAIL
  属于环境原因、不阻断放行）。

您**不需要** Flutter、源代码，也不需要连接构建服务器。本目录包含一切所需文件。

## 2. 目录内容

| 文件 | 大小（约） | 用途 |
|---|---|---|
| `bvi-kit-debug.apk` | ~180 MB | BVI Kit 调试包——所有演练/测试流程都用它 |
| `bvr-kit-debug.apk` | ~180 MB | BVR Kit 调试包 |
| `rts-kit-debug.apk` | ~160 MB | RTS Kit 调试包 |
| `bvi-kit-release.apk` | ~51 MB | BVI Kit 发布包（仅验证发布构建链路） |
| `bvr-kit-release.apk` | ~51 MB | BVR Kit 发布包 |
| `rts-kit-release.apk` | ~61 MB | RTS Kit 发布包（超预算 2%，已有说明） |
| `factory_test_harness.apk` | ~170 MB | 硬件测试 App（虹膜、指纹、打印机、短信、扫描、相机、GNSS、NTS、电池、USB、蓝牙、电话信息） |
| `factory_test.sh` | | 总控脚本：按顺序运行全部检查并输出 JSON 报告 |
| `smoke.sh` | | 冷启动 + 存活 + 无崩溃检查 |
| `kill-drill.sh` | | 在每个启动阶段强杀进程，验证恢复 |
| `offline-drill.sh` | | 飞行模式开关，验证 App 不崩溃 |
| `test_harness.sh` | | 运行硬件测试 App 并取回 JSON 报告 |
| `package_test_harness.sh` | | 重新打包 `factory_test_harness_package.zip` |
| `lib/common.sh`、`lib/device.sh` | | 公共库（必须与脚本放在一起） |
| `antiris/` | | 虹膜厂商库（设备侧依赖） |
| `test_harness/` | | 测试 App 源码（备查） |
| `README.md` | | 完整说明文档 |
| `factory_guidance.md` | | 发布前 APK 核查清单 |
| `compile_apks.md` | | APK 构建方法 + 可追溯信息（git 提交、SHA-256） |
| `SHA256SUMS` | | 上述所有文件的校验和 |
| `INSTALLER_TESTER_GUIDE.md` | | 本文件 |

**调试包 vs 发布包——装哪个？**

- 本指南的所有自动化测试：装**调试包**。只有 debuggable 构建才能用
  `run-as` 读出测试报告。
- 发布包用于验证优化后（AOT）构建能启动运行。但它们是用**调试密钥签名
  的**——不是正式生产包，**绝不能**进 MDM/生产环境。详见 `compile_apks.md`。

## 3. 工作站准备

1. **Android SDK Platform Tools**（`adb`）。检查：`adb --version`。
2. USB 线 + 一台打开 **USB 调试**的设备（设置 → 开发者选项 → USB 调试）。
   必须在设备上点“允许 USB 调试”，否则设备显示 `unauthorized`，所有脚本都会失败。
3. `bash`、`curl`、`shasum`（或 `sha256sum`）、`unzip`——macOS/Linux 一般自带。
4. 除非要从源码重新构建，否则不需要 Flutter。

确认设备可见：

```bash
adb devices
# emulator-5554	device     <- 序列号 + "device" 表示就绪
```

## 4. 先校验完整性（每台新工作站都要做）

```bash
cd factory_pack
shasum -c SHA256SUMS
# 每一行都必须以 OK 结尾；出现 FAILED 就重新拷贝整个包，不要测试
```

## 5. 安装 App

```bash
# 调试包（用于测试）。-g 会预授权，自动化运行时不会被系统弹窗卡住
# （rts-kit 启动时会申请相机 + 麦克风权限）。
adb -s <序列号> install -r -g bvi-kit-debug.apk
```

把 `<序列号>` 换成 `adb devices` 看到的串号，把 APK 换成要测的 App。
显示 `Success` 即安装成功。

## 6. 运行全套自动化测试（推荐）

```bash
./factory_test.sh bvi-kit --device <序列号>
```

脚本会自动安装同目录下对应的 `-debug.apk`，然后按顺序运行：
**smoke → kill-drill → offline-drill → test_harness**。遇到第一个失败就停止，
并生成 `factory-report-<应用>-<序列号>.json`，例如：

```json
{"app":"bvi-kit","device":"emulator-5554",
 "results":{"smoke.sh":"PASS","kill-drill.sh":"PASS",
            "offline-drill.sh":"PASS","test_harness.sh":"FAIL"},
 "result":"FAIL"}
```

全程约 6–8 分钟（主要是硬件 App 约 2 分钟的等待）。

也可以单独运行某一步：

```bash
./smoke.sh bvi-kit --device <序列号>            # 约 40 秒
./kill-drill.sh bvi-kit --device <序列号>       # 约 3 分钟，共 7 个阶段
./offline-drill.sh bvi-kit --device <序列号>    # 约 30 秒
./test_harness.sh ignore --device <序列号>      # 约 3 分钟
```

可选环境变量（运行前 export）：

| 变量 | 作用 |
|---|---|
| `EEL_TELEMETRY_URL` | smoke 测试向该地址探测会话（例如配合 `adb reverse tcp:9090 tcp:9090` 使用 `http://localhost:9090/healthz`）。未设置则遥测记为 SKIP（绝不伪造）。 |
| `EEL_DEDUP_CHECK` | 恢复网络后验证“精确一次”投递的 shell 命令（例如查询后端审计日志），必须以 exit 0 结束。未设置则去重记为 SKIP（绝不悄悄通过）。 |

## 7. 如何判读结果（重要——不是所有 FAIL 都阻断放行）

每步都会输出一行 JSON，含 `"result":"PASS"` 或 `"FAIL"` 及原因。状态含义：

| 状态 | 含义 | 是否阻断放行 |
|---|---|---|
| PASS | 检查真正通过 | 否 |
| FAIL | 检查真正失败——看 `reason` | **是**，除非下表列为环境原因 |
| SKIP | 前置条件缺失（遥测地址/去重钩子/硬件插件），会明确报告，绝不默认通过 | 否，但要记录 |
| NOT-RUN | 前一步失败，本步没跑 | 先修好前面的失败 |

**预期内的环境性结果（遇到不要阻断放行）：**

| 位置 | 结果 | 原因 | 做法 |
|---|---|---|---|
| 模拟器，硬件测试 | GNSS → FAIL | 室内/模拟器无卫星定位（30 秒超时是设计行为） | 在有天空视野的真机上会 PASS |
| 无 NTS 服务器的环境 | NTS → ERROR | `time.eel.local` 不可达 | 在有 NTS 服务的环境会 PASS |
| 没接相应硬件 | 虹膜、指纹、打印机、扫描仪 → SKIP（`No ... hardware attached`）| 2026-09-18 起插件已打包；SKIP 表示没外设，不是缺代码 | 接上外设会 PASS（虹膜需 AntIris UVC 相机；指纹/扫描仪还需厂商 SDK——见 ENGINEERING_FAQ）|
| 模拟器虚拟相机/无线电 | 相机、短信 → PASS | 经模拟器虚拟设备真实抓拍/发短信 | 证明驱动可用；在真机上复验 |
| 未设 `EEL_TELEMETRY_URL` | telemetry → SKIP | 没有可探测的地址 | 接后端联调时配置该变量 |
| 未设 `EEL_DEDUP_CHECK` | dedup → SKIP | 没有后端钩子 | 放行测试时提供钩子 |
| 模拟器冷启动 > 2000 ms | 仅 WARN | 2 秒预算针对实体中端机，不是模拟器 | 在基准真机上复测 |

**永远阻断放行的结果：**任何 `no-process`、`crash`、`offline-crash`、
`reconnect-death` FAIL；kill-drill 任一阶段 FAIL；硬件能力具备时取不到
`test_report.json`。

## 8. 手动读取硬件测试报告

```bash
adb -s <序列号> install -r -g factory_test_harness.apk
adb -s <序列号> shell monkey -p com.eel.test_harness -c android.intent.category.LAUNCHER 1
# 等约 2 分钟（9 项硬件测试，GNSS 一项就可能花 30 秒）
adb -s <序列号> shell run-as com.eel.test_harness cat app_flutter/test_report.json > test_report.json
```

每条记录含 `feature`、`status`（PASS/FAIL/SKIP/ERROR）、`details`、
`durationMs`。注意路径是 `app_flutter/`，不是 `files/`。

## 9. 故障排查

| 现象 | 原因 | 办法 |
|---|---|---|
| `device ... not online` / `adb devices` 为空 | 数据线、USB 调试没开、未授权 | 重插线、打开 USB 调试、在设备上点允许 |
| `no installed package matches 'xyz'` | 应用名拼错或没安装 | 核对名字（`bvi-kit`、`bvr-kit`、`rts-kit`）；先安装 |
| `ambiguous fragment` | 两个包都匹配 | 卸载旧包（`adb uninstall <包名>`） |
| 屏幕上卡着系统权限弹窗，测试不动 | App 首次启动申请权限（rts-kit：相机 + 麦克风） | 用 `-g` 重装；或 `adb shell pm grant <包名> <权限>` |
| 脚本无 FAIL 信息直接 exit 1 | 脚本版本太旧 | 重新拷贝整个包；不要手工改脚本 |
| `No test report retrieved` | 还没跑完（多等），或装的是发布版 harness | 等 2 分钟再取；harness 必须用调试版 APK |
| `run-as ... Permission denied` | 装的是不可调试的发布版 | 改装调试版 `factory_test_harness.apk` |
| `INSTALL_FAILED_NO_MATCHING_ABI` | ABI 不匹配（仅分 ABI 包会出现） | 用本包里的通用 `-debug.apk` |
| 模拟器冷启动 WARN 超 2000 ms | 模拟器慢，预算针对真机 | 在基准真机上复测 |
| `shasum -c` 出现 FAILED | 拷贝损坏/不完整 | 重新拷贝整个包，不要测试 |

## 10. 预算与铁律

- 冷启动到首帧：实体中端机上 **≤ 2000 ms**。
- APK 体积：普通应用 ≤ 40 MB，**套件类 ≤ 60 MB**（指发布包）。
- 调试包（~150–180 MB）豁免——里面是调试 JIT + 模拟器 ABI + 校验层，
  发布包里都没有。
- 这里的发布包是**调试密钥签名**：只用于验证发布链路。绝不能进
  MDM/生产环境。
- 不要手工改脚本；没有 §7 明确列出的环境原因，不要跳过任何 FAIL。

## 11. 重新打包分发 zip（维护人员）

```bash
./package_test_harness.sh --zip --include-src
shasum -c SHA256SUMS
```

这会重新构建调试版 harness、重新生成 `SHA256SUMS` 并输出
`factory_test_harness_package.zip`（源码只含 Android + 公共文件，不含
桌面/iOS 目录和 IDE 文件）。

# Engineering FAQ — Mass Production Readiness
# 工程常见问题 — 量产准备情况

> English first, 中文在后. Status of each answer: verified against the
> repository tree on 2026-09-18 unless marked as pending work.
> 每条回答的状态：除特别注明外，均已于 2026-09-18 对代码库实测验证。

---

# Part 1 — English

## Q1. Which packages need to be preinstalled for mass production? Does this include the Harness?

**Short answer: no system-image preloads are required. The Harness must NOT ship.**

- The kits (bvi-kit, bvr-kit, rts-kit) are ordinary installable APKs. Every
  dependency they currently use comes from pub Flutter plugins. Nothing
  needs to live in `/system/priv-app` today.
- Production devices get the kit APKs through **MDM** (managed Google Play /
  ABM path, see `operations/mdm/`), not baked into the factory image.
- The device hardware/kernel must provide **USB host (OTG)** — the iris
  camera and the SMS modem are USB peripherals. Known gap: kit manifests do
  not yet declare `<uses-feature android.hardware.usb.host>` (scheduled M2
  fix; harmless on capable hardware, but store filtering cannot verify
  compatibility without it).
- Antiris native libraries: see Q4.
- **The test harness (`factory_test_harness`) is factory-only.** It is
  debuggable, enumerates IMEI/USB/Bluetooth inventories, and requests
  CAMERA + MIC + location + phone-state permissions. It must never be staged
  to MDM or production devices — same as the `-debug.apk` kit builds.

## Q2. Can you provide a release-signed APK? The current builds are all debug.

**Not yet — and no one should improvise one.** Facts:

- The three `*-release.apk` files are genuine release artifacts (AOT-compiled,
  non-debuggable, `apksigner verify` green under the v2 scheme), but they are
  signed with the **debug key** — that is what
  `signingConfigs.getByName("debug")` in all three
  `android/app/build.gradle.kts` produces, and no `key.properties` exists
  anywhere in the repo.
- Production signing lives in OpenBao and is executed by `sign.sh` (M5);
  whoever owns the release keystore owns this step.

What *is* possible without rebuilding: the moment the release keystore (or
OpenBao access for `sign.sh`) is provided, the **existing AOT binaries can be
re-signed in place** (`apksigner sign --ks <release keystore>`), re-verified,
and re-checksummed — roughly a 10-minute task, byte-identical code under a
new signature. Minting any other "release-looking" key would be mislabeling
and will not be done.

## Q3. Will HALs such as iris/fingerprint, or Flutter plugins, be delivered separately? Otherwise the Harness items stay SKIP.

**Yes — separate delivery is required, and SKIP is currently the expected
result, not a failure.** Verified tree state:

- The Dart side defines only MethodChannel contracts plus simulated
  transports (`eel/iris`, `eel/fingerprint` in
  `packages/kit-sdk-flutter`; the harness registers no native handlers at
  all for printer, SMS, document scanner, or camera).
- Native code exists solely as **unwired reference files**:
  `services/biometric-edge-svc/android/.../AntIrisPlugin.kt` and
  `Fap60Plugin.kt`. That directory is not a Flutter plugin package and is not
  referenced by any app pubspec (bvi-kit depends only on `kit-sdk-flutter`).
- No printer, SMS-modem, scanner, or camera native implementation exists
  anywhere in the repository.

What has to arrive (M3 work): one **Flutter plugin package per peripheral**
(`android/` with jars + `jniLibs`, Dart API on the documented `eel/*`
channels), wired into the kit app pubspecs. Until then, Iris / Fingerprint /
Printer / SMS / Scanner / Camera → SKIP is correct behavior, and the factory
scripts score it as such.

**Update 2026-09-18 — first half landed as `packages/eel_drivers/`** (bundled
into all 7 shipped APKs, verified on emulator):

- Shipped and working: USB printer-class ESC/POS, SmsManager modem
  (SIM-ready + own-number gating, self-send only), Camera2 still capture
  (**PASS** on emulator virtual camera), AntIris USB stack (jars + both-ABI
  `.so` bundled; SKIP without UVC hardware).
- Documented slots (presence probes, honest SKIP): FAP60 fingerprint
  (**vendor SDK jars missing** — init reports `fap60-sdk-missing`; USB
  VID/PID 0x0507/0x0581 detection works) and document scanner (known VID/PID
  table empty; HID enumeration in probe details).
- Each channel serves `probeDetails` → `{present, reason, devices}` so the
  harness tells "no hardware" (SKIP) from "operation failed" (FAIL).
- Still fully outstanding: kit workflow integration (kit Dart drivers still
  use simulated transports) and the FAP60/scanner vendor SDKs.

**Bundling status (2026-09-18): all drivers are now bundled into all 7 APKs.**

New `packages/eel_drivers/` plugin (Dart + Kotlin, analyze-clean, unit test
green). One Android plugin class serving 7 channels (exact harness + kit-sdk
names, incl. the `eel/iris` alias). Every channel also serves `probeDetails`
→ `{present, reason, devices}`, so the harness distinguishes "no hardware"
(SKIP) from "operation failed" (FAIL):

| Driver | Implementation |
|---|---|
| Printer | USB printer-class ESC/POS (init + test page + cut over bulk-OUT) |
| SMS | SmsManager, SIM-ready + own-number gating, self-send only |
| Scanner | Vendor slot (empty VID/PID table, HID enumeration in details) |
| Camera | Camera2 still capture, back-preferred, 10 s bound |
| Iris | Ported AntIris stack (USBControl + IrisIso), vendor jars + both-ABI .so bundled |
| Fingerprint | FAP60 VID/PID probe; init reports `fap60-sdk-missing` (vendor jars absent) |

Wired into test_harness + bvi/bvr/rts kits; verified `com/eel/drivers` refs
and `lib_Antiris`/`libUSBCamera` in all 7 APKs; all 3 kits + harness smoke
PASS on the emulator.

Notes that narrow the scope:

- FAP60 is an **external USB** scanner, so no OS fingerprint HAL is needed —
  USB host support plus the vendor plugin is sufficient.
- The OS camera HAL is standard and already exercised (rts-kit declares
  CAMERA + RECORD_AUDIO via plugin-contributed manifest entries).

## Q4. Should Antiris go into the APK or into /system/lib64?

**Recommendation: bundle it in the APK.** Grounds, all repo-verified:

- The plan already assumes APK bundling:
  `docs/mobile/M3_HARDWARE_INTEGRATION.md` specifies
  `libs/antiris.antusbcamera.jar` packaging, and the vendor SDK
  (`sdks/AntIris_Android_SDK/libs/`) ships both ABIs (`armeabi-v7a`,
  `arm64-v8a`) exactly for app-side bundling.
- Cost is small (~4–6 MB per ABI set: `antiris.antusbcamera.jar` +
  `serenegiant.jar` ~1 MB, plus `lib_Antiris` / `libUSBCamera` /
  `libopenjpeg` / `libpng`): bvi-kit release would move ~51 → ~57 MB,
  still inside the 60 MB kit budget.
- It preserves the standard MDM story: no system-image fork, no signed OTA
  needed to update the iris stack, the driver versions together with the app
  that uses it, and loading via `System.loadLibrary` from the app lib
  directory needs no privilege.
- `/system/lib64` only wins if the stack must be shared across multiple
  independent apps with version lockstep, or must survive factory reset
  without an MDM re-push. Neither applies today.

Two caveats before executing:

1. **Verify the vendor license first.** No license file was found next to the
   SDK — only API docs in English and Chinese
   (`AntIris_Android_SDK_en.pdf` / `AntIris_Android_SDK_zh.pdf`; the Chinese
   doc may help vendor conversations). Confirm in-app redistribution is
   permitted before bundling.
2. **APKs now contain the Antiris bytes** (verified by archive listing since
  the 2026-09-18 `eel_drivers` bundling) — but iris capture still needs
  attached UVC hardware plus the M3 workflow integration.

---

## Q5. Is there a generic driver we could use for the printer?

**Yes — and it is already bundled.** What ships in the APKs today is the
generic driver for the most common case:

- **ESC/POS over USB** (`eel_drivers` PrinterHandler): talks to any USB
  printer-class (7) device — Epson, XPrinter, Sunmi, and the many clones
  that emulate ESC/POS — init + test page + cut over bulk-OUT. Notably, the
  module spec (`docs/mobile/bvi-kit.md`) already calls for exactly this:
  *"ESC/POS over USB/BT platform channel."* The USB half is done.

What "generic" can and cannot cover from here depends on the printer class,
and no model is named anywhere in the repo:

| Printer class | Generic path | Status |
|---|---|---|
| Thermal receipt, USB | ESC/POS (bundled) | ✅ done |
| Thermal receipt, Bluetooth | Same ESC/POS bytes over an SPP socket — small addition to the same handler | ⬜ needs a BT printer to validate against |
| Label printers (e.g. Zebra) | **ZPL, not ESC/POS** — an entirely different language; needs a ZPL formatter added alongside | ⬜ needs the model confirmed (Zebra appears in-repo only for RFID readers, not printers) |
| Office/laser over WiFi | No driver needed — Android's built-in Print framework (`PrintManager`, PDF via `PrintDocumentAdapter`) handles it with zero native code | ⬜ ~1 day, no hardware dependency |
| Network raw (port 9100) | Plain TCP socket + ESC/POS or ZPL payload | ⬜ trivial once the language is known |

Bottom line: the generic USB tier is covered. The one question that unlocks
the rest is the same as for the scanner — **which printer make/model will
the factory use?** A USB/Bluetooth thermal unit means we are likely already
done (or one SPP transport short of done); Zebra label stock means adding a
ZPL formatter + template path next.

---

# Part 2 — 中文 (简体中文)

## Q1. 量产需要在系统中预装哪些包？包括测试 Harness 吗？

**简短回答：目前不需要预装任何系统包。Harness 绝不能出货。**

- 三个套件 App（bvi-kit、bvr-kit、rts-kit）都是普通可安装 APK，现有依赖
  全部来自 pub Flutter 插件。今天没有任何东西需要放进
  `/system/priv-app`。
- 量产设备通过 **MDM** 获取套件 APK（managed Google Play / ABM 通道，
  见 `operations/mdm/`），而不是烤进出厂镜像。
- 设备硬件/内核必须支持 **USB host（OTG）**——虹膜摄像头和短信 modem
  都是 USB 外设。已知缺口：套件 manifest 尚未声明
  `<uses-feature android.hardware.usb.host>`（已排入 M2 修复；在有能力的
  硬件上无影响，但没有它就无法做商店兼容性过滤）。
- Antiris 原生库：见 Q4。
- **测试 Harness（`factory_test_harness`）仅限工厂使用。**它是 debuggable
  构建，会枚举 IMEI/USB/蓝牙信息，并申请相机 + 麦克风 + 定位 + 电话状态
  权限。绝不能进 MDM 或量产设备——和 `-debug.apk` 套件包一样。

## Q2. 能提供 release 签名的 APK 吗？现在的构建全是 debug 的。

**暂时不能——而且任何人都不该自行“造”一个。**事实：

- 三个 `*-release.apk` 是真正的发布构建（AOT 编译、不可调试、
  `apksigner verify` v2 验证通过），但用的是**调试密钥**签名——三个
  `android/app/build.gradle.kts` 里写的就是
  `signingConfigs.getByName("debug")`，仓库里也不存在任何
  `key.properties`。
- 正式签名在 OpenBao 里，由 `sign.sh` 执行（M5 工作），谁持有 release
  keystore，谁才能做这一步。

不需要重新构建即可做的事：一旦提供 release keystore（或 `sign.sh`
所需的 OpenBao 权限），**现有 AOT 二进制可以直接重签**
（`apksigner sign --ks <release keystore>`），再验证、再生成校验和——
约 10 分钟的工作量，代码逐字节不变，只换签名。其他任何“看起来像
release”的密钥都属于贴假标签，不会做。

## Q3. 虹膜/指纹等 HAL 或 Flutter 插件会单独交付吗？否则 Harness 相关项就一直是 SKIP。

**是的，必须单独交付；目前的 SKIP 是预期结果，不是故障。**已验证的代码现状：

- Dart 侧只有 MethodChannel 契约加模拟传输（`packages/kit-sdk-flutter`
  里的 `eel/iris`、`eel/fingerprint`；Harness 对打印机、短信、扫描仪、
  相机根本没有注册任何原生处理器）。
- 原生代码仅以**未接入的参考文件**形式存在：
  `services/biometric-edge-svc/android/.../AntIrisPlugin.kt` 和
  `Fap60Plugin.kt`。该目录不是 Flutter 插件包，也没有任何 App 的
  pubspec 引用它（bvi-kit 只依赖 `kit-sdk-flutter`）。
- 仓库里不存在打印机、短信 modem、扫描仪、相机的任何原生实现。

必须到货的东西（M3 工作）：**每个外设一个 Flutter 插件包**
（`android/` 含 jar + `jniLibs`，Dart 侧按文档的 `eel/*` 通道封装），
再接入各套件 App 的 pubspec。在此之前，虹膜 / 指纹 / 打印机 / 短信 /
扫描仪 / 相机 → SKIP 是正确行为，工厂脚本也按此计分。

**2026-09-18 更新——第一批已落地为 `packages/eel_drivers/`**（已打进全部
7 个出货 APK，模拟器验证通过）：

- 已出货且可用：USB 打印机类 ESC/POS、SmsManager modem（SIM 就绪 + 本机
  号码门控，只给自己发）、Camera2 静态抓拍（模拟器虚拟相机上 **PASS**）、
  AntIris USB 栈（jar + 双 ABI `.so` 已打包；无 UVC 硬件时 SKIP）。
- 已占位（ presence 探测，诚实 SKIP）：FAP60 指纹（**缺厂商 SDK jar 包**——
  init 直接报告 `fap60-sdk-missing`；USB VID/PID 0x0507/0x0581 探测可用）
  和文档扫描仪（已知 VID/PID 表为空；probe 详情里枚举 HID 设备）。
- 每个通道都提供 `probeDetails` → `{present, reason, devices}`，Harness
  能区分“没硬件”（SKIP）和“操作失败”（FAIL）。
- 仍完全未做：套件工作流接入（套件 Dart 驱动仍用模拟传输）和
  FAP60/扫描仪厂商 SDK。

**打包状态（2026-09-18）：全部驱动已打进全部 7 个 APK。**

新的 `packages/eel_drivers/` 插件（Dart + Kotlin，analyze干净，单元测试
通过）。一个 Android 插件类服务 7 个通道（与 Harness + kit-sdk 的通道名
完全一致，含 `eel/iris` 别名）。每个通道还提供 `probeDetails` →
`{present, reason, devices}`，Harness 能区分“没硬件”（SKIP）和“操作
失败”（FAIL）：

| 驱动 | 实现 |
|---|---|
| 打印机 | USB 打印机类 ESC/POS（初始化 + 测试页 + 切刀，经 bulk-OUT） |
| 短信 | SmsManager，SIM 就绪 + 本机号码门控，只给自己发 |
| 扫描仪 | 厂商占位（VID/PID 表为空，详情里枚举 HID） |
| 相机 | Camera2 静态抓拍，后置优先，10 秒超时 |
| 虹膜 | 移植的 AntIris 栈（USBControl + IrisIso），厂商 jar + 双 ABI `.so` 已打包 |
| 指纹 | FAP60 VID/PID 探测；init 报告 `fap60-sdk-missing`（缺厂商 jar 包） |

已接入 test_harness + bvi/bvr/rts 三个套件；7 个 APK 里验明
`com/eel/drivers` 引用和 `lib_Antiris`/`libUSBCamera`；三个套件 +
Harness 在模拟器上 smoke 全部 PASS。

两点缩小范围的说明：

- FAP60 是**外置 USB**指纹仪，不需要 OS 指纹 HAL——USB host 加厂商插件
  就够了。
- OS 相机 HAL 是标准的，且已被实际行使（rts-kit 通过插件贡献的
  manifest 条目声明了 CAMERA + RECORD_AUDIO）。

## Q4. Antiris 应该打进 APK，还是放 /system/lib64？

**建议：打进 APK。**依据全部来自仓库实测：

- 方案文档本来就是这么定的：
  `docs/mobile/M3_HARDWARE_INTEGRATION.md` 明确写了
  `libs/antiris.antusbcamera.jar` 的打包方式；厂商 SDK
  （`sdks/AntIris_Android_SDK/libs/`）同时提供 `armeabi-v7a` 和
  `arm64-v8a` 两个 ABI，正是为 App 侧打包准备的。
- 代价很小（每套 ABI 约 4–6 MB：两个 jar 约 1 MB，加上
  `lib_Antiris` / `libUSBCamera` / `libopenjpeg` / `libpng`）：bvi-kit
  发布包约从 51 MB 涨到 57 MB，仍在 60 MB 套件预算内。
- 保持标准 MDM 链路：不用 fork 系统镜像，升级虹膜栈不需要签 OTA，
  驱动和用它的 App 同版本发布，用 `System.loadLibrary` 从 App lib
  目录加载不需要任何特权。
- `/system/lib64` 只有在以下情况才更优：多个独立 App 必须共享同一套栈
  且版本锁定，或者要求恢复出厂设置后不经 MDM 重装依然可用。今天两条都不成立。

动手前两个注意事项：

1. **先确认厂商授权。**SDK 旁边没找到 license 文件，只有中英文 API
   文档（`AntIris_Android_SDK_en.pdf` / `AntIris_Android_SDK_zh.pdf`，
   中文文档可用于和厂商沟通）。打包前请确认允许在 App 内再分发。
2. **APK 里已有 Antiris 内容**（2026-09-18 `eel_drivers` 打包后已逐包
   验明）——但虹膜采集仍需外接 UVC 硬件，外加 M3 工作流接入。

---

## Q5. 打印机有没有现成的通用驱动可以用？

**有——而且已经打进包了。**现在 APK 里装的就是最常见情况的通用驱动：

- **USB ESC/POS**（`eel_drivers` PrinterHandler）：和任何 USB 打印机类（7）
  设备通信——Epson、XPrinter、Sunmi，以及大量兼容 ESC/POS 的机型——经
  bulk-OUT 做初始化 + 测试页 + 切刀。值得注意的是，模块规格
  （`docs/mobile/bvi-kit.md`）本来就要的就是这个：*“ESC/POS over USB/BT
  platform channel.”* USB 一半已经完成。

往下“通用”能覆盖多少取决于打印机类型，而仓库里没有点名任何型号：

| 打印机类型 | 通用路径 | 状态 |
|---|---|---|
| 热敏小票，USB | ESC/POS（已打包） | ✅ 完成 |
| 热敏小票，蓝牙 | 同样的 ESC/POS 字节走 SPP 通道——在同一个 handler 上加一小段 | ⬜ 需要一台蓝牙打印机联调验证 |
| 标签打印机（如 Zebra） | **ZPL，不是 ESC/POS**——完全是另一种语言；需要并排加一个 ZPL 排版器 | ⬜ 需先确认型号（Zebra 在仓库里只出现在 RFID 读写器，和打印机无关） |
| 办公室激光打印机，WiFi | 不需要驱动——Android 内建打印框架（`PrintManager`，经 `PrintDocumentAdapter` 打 PDF）零原生代码搞定 | ⬜ 约 1 天，不依赖硬件 |
| 网络 raw（9100 端口） | 普通 TCP socket + ESC/POS 或 ZPL 负载 | ⬜ 语言确定后很简单 |

结论：通用 USB 层已经覆盖。解锁剩下部分的关键问题和扫描仪一样——
**工厂到底用哪款打印机？**如果是 USB/蓝牙热敏机，很可能已经完成
（或只差一个 SPP 传输层）；如果是 Zebra 标签纸，下一步就加 ZPL 排版器
+ 模板路径。

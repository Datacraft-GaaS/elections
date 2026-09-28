// SPDX-License-Identifier: Apache-2.0
//
// Thin RAII wrappers over the eel-bvr C ABI.
//
// The C functions have two obligations the C++ side must not forget: every
// returned `char *` is owned by Rust and must go through `eel_string_free`, and
// every call can fail into a caller-owned error string. Wrapping them here means
// the window code below deals in `std::optional<std::string>` and never
// hand-frees anything, which is where boundary bugs normally live.

#pragma once

#include <QJsonArray>
#include <QJsonObject>
#include <QString>

#include <memory>
#include <optional>
#include <string>
#include <vector>

extern "C" {
#include "eel_bvr.h"
}

namespace eel {

/// Frees a Rust-owned string exactly once.
struct CStringDeleter {
    void operator()(char *p) const noexcept {
        if (p) {
            eel_string_free(p);
        }
    }
};

using OwnedString = std::unique_ptr<char, CStringDeleter>;

/// Takes ownership of a returned C string and converts it to a Qt string.
inline std::optional<QString> takeString(char *raw) {
    if (!raw) {
        return std::nullopt;
    }
    OwnedString owned(raw);
    return QString::fromUtf8(owned.get());
}

/// A compiled workflow, with the handle owned for its lifetime.
class Workflow {
public:
    /// Compiles `document`. On failure `error` carries the compiler's message,
    /// which for a document that parses is a JSON array of every fault found.
    static std::optional<Workflow> compile(const QString &document, QString *error);

    ~Workflow();
    /// Move-only: the handle is owned and must not be duplicated.
    Workflow(Workflow &&other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
    Workflow &operator=(Workflow &&other) noexcept;
    Workflow(const Workflow &) = delete;
    Workflow &operator=(const Workflow &) = delete;

    QString id() const;
    int stepCount() const;
    QString stepLabel(int index) const;
    /// 0 data entry, 1 capture, 2 review, 3 submit.
    int stepKind(int index) const;
    /// Field descriptors for a data-entry step, in form order.
    QList<QJsonObject> stepFields(int index) const;
    /// The capture requirement for a capture step.
    QJsonObject captureRequirement(int index) const;

    /// The underlying handle, for the free function `validate` below.
    EelWorkflow *rawHandle() const { return handle_; }

private:
    explicit Workflow(EelWorkflow *handle) : handle_(handle) {}
    EelWorkflow *handle_;
};

/// The offline record store for this station.
class Store {
public:
    /// Creates a store with a fresh instance identity.
    static std::optional<Store> create(const QString &stationLabel, QString *error);

    ~Store();
    /// Move-only: the handle is owned and must not be duplicated.
    Store(Store &&other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }
    Store &operator=(Store &&other) noexcept;
    Store(const Store &) = delete;
    Store &operator=(const Store &) = delete;

    QString instanceId() const;
    QString publicKeyBase64() const;

    /// Appends a captured value. Returns the new record id, or nullopt on error.
    std::optional<QString> append(const QString &dataItem,
                                  const QString &dataClass,
                                  const QString &valueJson,
                                  const QString &operatorId,
                                  const QString &workflowId,
                                  const QString &stepId,
                                  const QString &purpose,
                                  const QString &captureMethod,
                                  QString *error);

    /// Decrypts a stored value.
    std::optional<QString> value(const QString &recordId, QString *error);

    /// Runs the integrity check. Returns true when the store is intact.
    bool verify(QString *error) const;

    /// The pending sync batch as JSON, or nullopt when nothing is pending.
    QJsonObject pendingBatch(QString *error) const;

    /// Progress as {total, synced, pending}.
    QJsonObject syncState() const;

    /// Marks records acknowledged; returns how many newly changed, or -1.
    int markSynced(const QStringList &recordIds, QString *error);

    /// Writes the store to `path`, atomically.
    ///
    /// `keyEncryptionKey` must be 32 bytes. The data key is sealed under it and
    /// never written in the clear, so a deployed station supplies a TPM- or
    /// HSM-backed key. A build using a software keystore supplies bytes it
    /// already holds, which protects the file against casual reading but not
    /// against an attacker who has those bytes.
    bool sealToDisk(const QString &path, const QByteArray &keyEncryptionKey, QString *error);

    /// Returns `true` when `publicKey` is the key this store signs with.
    ///
    /// Used after restoring a store to fail fast, rather than discovering at
    /// verify time that every record is unreadable.
    bool checkKey(const QByteArray &publicKey) const;

private:
    explicit Store(EelStore *handle) : handle_(handle) {}
    EelStore *handle_;
};

/// Validates `submission` against `workflow`.
///
/// Returns an empty vector when valid. Each violation is {code, message}, where
/// `code` is stable and localisable.
std::vector<QJsonObject> validate(const Workflow &workflow,
                                  const QJsonObject &submission,
                                  const QStringList &capturedKinds,
                                  QString *error);

} // namespace eel

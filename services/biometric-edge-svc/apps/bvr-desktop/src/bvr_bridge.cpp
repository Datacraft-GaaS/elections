// SPDX-License-Identifier: Apache-2.0
//
// Implementation of the RAII wrappers declared in bvr_bridge.h.

#include "bvr_bridge.h"

#include <QJsonArray>
#include <QJsonDocument>

namespace eel {
namespace {

/// Copies a raw C string into the caller's `error` out-parameter slot, and
/// frees it. The C ABI uses this out-parameter convention rather than a return
/// code for messages.
void takeError(char *raw, QString *error) {
    if (raw && error) {
        *error = QString::fromUtf8(raw);
    }
    if (raw) {
        eel_string_free(raw);
    }
}

/// Parses a JSON document, returning an empty object on failure.
QJsonObject parseObject(const QString &text) {
    QJsonParseError parseError{};
    const QJsonDocument doc = QJsonDocument::fromJson(text.toUtf8(), &parseError);
    if (parseError.error != QJsonParseError::NoError || !doc.isObject()) {
        return {};
    }
    return doc.object();
}

} // namespace

// --------------------------------------------------------------- Workflow --

std::optional<Workflow> Workflow::compile(const QString &document, QString *error) {
    char *rawError = nullptr;
    const QByteArray utf8 = document.toUtf8();
    EelWorkflow *handle =
        eel_workflow_load(utf8.constData(), &rawError);
    if (!handle) {
        takeError(rawError, error);
        return std::nullopt;
    }
    takeError(rawError, error);
    return Workflow(handle);
}

Workflow::~Workflow() {
    if (handle_) {
        eel_workflow_free(handle_);
    }
}

Workflow &Workflow::operator=(Workflow &&other) noexcept {
    if (this != &other) {
        if (handle_) {
            eel_workflow_free(handle_);
        }
        handle_ = other.handle_;
        other.handle_ = nullptr;
    }
    return *this;
}

QString Workflow::id() const {
    return takeString(eel_workflow_id(handle_)).value_or(QString());
}

int Workflow::stepCount() const {
    return static_cast<int>(eel_workflow_step_count(handle_));
}

QString Workflow::stepLabel(int index) const {
    char *rawError = nullptr;
    const std::optional<QString> label =
        takeString(eel_workflow_step_label(handle_, static_cast<size_t>(index), &rawError));
    if (rawError) {
        eel_string_free(rawError);
    }
    return label.value_or(QString());
}

int Workflow::stepKind(int index) const {
    char *rawError = nullptr;
    const int kind =
        eel_workflow_step_kind(handle_, static_cast<size_t>(index), &rawError);
    if (rawError) {
        eel_string_free(rawError);
    }
    return kind;
}

QList<QJsonObject> Workflow::stepFields(int index) const {
    QList<QJsonObject> fields;
    char *rawError = nullptr;
    const size_t count =
        eel_workflow_step_field_count(handle_, static_cast<size_t>(index), &rawError);
    if (rawError) {
        eel_string_free(rawError);
    }
    for (size_t i = 0; i < count; ++i) {
        char *fieldError = nullptr;
        const std::optional<QString> json = takeString(eel_workflow_field_json(
            handle_, static_cast<size_t>(index), i, &fieldError));
        if (fieldError) {
            eel_string_free(fieldError);
        }
        if (!json) {
            continue;
        }
        // Preserve field order: the form must read in the document's order.
        QJsonObject object = parseObject(*json);
        object.insert(QStringLiteral("_order"), static_cast<int>(i));
        fields.append(object);
    }
    return fields;
}

QJsonObject Workflow::captureRequirement(int index) const {
    char *rawError = nullptr;
    const std::optional<QString> json =
        takeString(eel_workflow_capture_json(handle_, static_cast<size_t>(index), &rawError));
    if (rawError) {
        eel_string_free(rawError);
    }
    if (!json) {
        return {};
    }
    return parseObject(*json);
}

// ------------------------------------------------------------------ Store --

std::optional<Store> Store::create(const QString &stationLabel, QString *error) {
    char *rawError = nullptr;
    const QByteArray utf8 = stationLabel.toUtf8();
    EelStore *handle = eel_store_new(utf8.constData(), &rawError);
    if (!handle) {
        takeError(rawError, error);
        return std::nullopt;
    }
    takeError(rawError, error);
    return Store(handle);
}

Store::~Store() {
    if (handle_) {
        eel_store_free(handle_);
    }
}

Store &Store::operator=(Store &&other) noexcept {
    if (this != &other) {
        if (handle_) {
            eel_store_free(handle_);
        }
        handle_ = other.handle_;
        other.handle_ = nullptr;
    }
    return *this;
}

QString Store::instanceId() const {
    return takeString(eel_store_instance_id(handle_)).value_or(QString());
}

QString Store::publicKeyBase64() const {
    return takeString(eel_store_public_key(handle_)).value_or(QString());
}

std::optional<QString> Store::append(const QString &dataItem,
                                     const QString &dataClass,
                                     const QString &valueJson,
                                     const QString &operatorId,
                                     const QString &workflowId,
                                     const QString &stepId,
                                     const QString &purpose,
                                     const QString &captureMethod,
                                     QString *error) {
    const QByteArray item = dataItem.toUtf8();
    const QByteArray klass = dataClass.toUtf8();
    const QByteArray value = valueJson.toUtf8();
    const QByteArray oper = operatorId.toUtf8();
    const QByteArray flow = workflowId.toUtf8();
    const QByteArray step = stepId.toUtf8();
    const QByteArray why = purpose.toUtf8();
    const QByteArray how = captureMethod.toUtf8();

    char *outId = nullptr;
    char *rawError = nullptr;
    const int rc = eel_store_append(handle_, item.constData(), klass.constData(),
                                    value.constData(), oper.constData(), flow.constData(),
                                    step.constData(), why.constData(), how.constData(),
                                    &outId, &rawError);
    if (rc != 0) {
        takeError(rawError, error);
        if (outId) {
            eel_string_free(outId);
        }
        return std::nullopt;
    }
    takeError(rawError, error);
    const std::optional<QString> id = takeString(outId);
    if (!id) {
        return std::nullopt;
    }
    return id;
}

std::optional<QString> Store::value(const QString &recordId, QString *error) {
    const QByteArray id = recordId.toUtf8();
    char *rawError = nullptr;
    const std::optional<QString> json =
        takeString(eel_store_value(handle_, id.constData(), &rawError));
    if (rawError) {
        takeError(rawError, error);
        return std::nullopt;
    }
    if (rawError) {
        eel_string_free(rawError);
    }
    return json;
}

bool Store::verify(QString *error) const {
    char *rawError = nullptr;
    const int rc = eel_store_verify(handle_, &rawError);
    if (rc == 0) {
        if (rawError) {
            eel_string_free(rawError);
        }
        return true;
    }
    takeError(rawError, error);
    return false;
}

QJsonObject Store::pendingBatch(QString *error) const {
    char *rawError = nullptr;
    const std::optional<QString> json =
        takeString(eel_store_pending_batch(handle_, &rawError));
    if (!json) {
        takeError(rawError, error);
        return {};
    }
    if (rawError) {
        eel_string_free(rawError);
    }
    return parseObject(*json);
}

QJsonObject Store::syncState() const {
    char *rawError = nullptr;
    const std::optional<QString> json = takeString(eel_store_sync_state(handle_, &rawError));
    if (rawError) {
        eel_string_free(rawError);
    }
    if (!json) {
        return {};
    }
    return parseObject(*json);
}

int Store::markSynced(const QStringList &recordIds, QString *error) {
    std::vector<QByteArray> owned;
    owned.reserve(static_cast<size_t>(recordIds.size()));
    for (const QString &id : recordIds) {
        owned.push_back(id.toUtf8());
    }
    std::vector<const char *> pointers;
    pointers.reserve(owned.size());
    for (const QByteArray &id : owned) {
        pointers.push_back(id.constData());
    }

    char *rawError = nullptr;
    size_t marked = 0;
    const int rc = eel_store_mark_synced(handle_,
                                         pointers.empty() ? nullptr : pointers.data(),
                                         pointers.size(), &marked, &rawError);
    if (rc != 0) {
        takeError(rawError, error);
        return -1;
    }
    if (rawError) {
        eel_string_free(rawError);
    }
    return static_cast<int>(marked);
}

bool Store::sealToDisk(const QString &path, const QByteArray &keyEncryptionKey, QString *error) {
    if (keyEncryptionKey.size() != 32) {
        if (error) {
            *error = QStringLiteral("key-encryption key must be 32 bytes, got %1")
                         .arg(keyEncryptionKey.size());
        }
        return false;
    }
    const QByteArray pathUtf8 = path.toUtf8();
    char *rawError = nullptr;
    const int rc = eel_store_seal_to_disk(handle_, pathUtf8.constData(),
                                          reinterpret_cast<const unsigned char *>(
                                              keyEncryptionKey.constData()),
                                          &rawError);
    if (rc != 0) {
        takeError(rawError, error);
        return false;
    }
    if (rawError) {
        eel_string_free(rawError);
    }
    return true;
}

bool Store::checkKey(const QByteArray &publicKey) const {
    if (publicKey.size() != 32) {
        return false;
    }
    char *rawError = nullptr;
    const int rc = eel_store_check_key(
        handle_, reinterpret_cast<const unsigned char *>(publicKey.constData()), &rawError);
    if (rawError) {
        eel_string_free(rawError);
    }
    return rc == 0;
}

// -------------------------------------------------------------- validate --

std::vector<QJsonObject> validate(const Workflow &workflow,
                                  const QJsonObject &submission,
                                  const QStringList &capturedKinds,
                                  QString *error) {
    std::vector<QJsonObject> violations;
    const QByteArray submissionJson =
        QJsonDocument(submission).toJson(QJsonDocument::Compact);
    const QByteArray capturedJson =
        QJsonDocument(QJsonArray::fromStringList(capturedKinds)).toJson(QJsonDocument::Compact);

    char *outViolations = nullptr;
    char *rawError = nullptr;
    const int rc = eel_workflow_validate(workflow.rawHandle(),
                                         submissionJson.constData(),
                                         capturedJson.constData(),
                                         &outViolations, &rawError);
    if (rc < 0) {
        takeError(rawError, error);
        if (outViolations) {
            eel_string_free(outViolations);
        }
        return violations;
    }
    if (rawError) {
        eel_string_free(rawError);
    }
    if (!outViolations) {
        return violations;
    }
    const std::optional<QString> json = takeString(outViolations);
    if (!json) {
        return violations;
    }
    QJsonParseError parseError{};
    const QJsonDocument doc = QJsonDocument::fromJson(json->toUtf8(), &parseError);
    if (parseError.error != QJsonParseError::NoError) {
        return violations;
    }
    for (const QJsonValue &value : doc.array()) {
        if (value.isObject()) {
            violations.push_back(value.toObject());
        }
    }
    return violations;
}

} // namespace eel

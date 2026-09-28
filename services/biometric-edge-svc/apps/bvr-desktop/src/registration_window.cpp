// SPDX-License-Identifier: Apache-2.0
//
// The BVR registration window.
//
// The form is generated from the workflow document, not hand-built. That is the
// whole point of the DSL: a new data item is an edit to a JSON document, not a
// code change and a release. This window asks the compiled workflow what to show
// and renders it.

#include "registration_window.h"

#include <QCheckBox>
#include <QComboBox>
#include <QDateEdit>
#include <QFormLayout>
#include <QGroupBox>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QMessageBox>
#include <QPushButton>
#include <QScrollArea>
#include <QStackedWidget>
#include <QStatusBar>
#include <QTextEdit>
#include <QVBoxLayout>

namespace eel {
namespace {

} // namespace

RegistrationWindow::RegistrationWindow(Workflow workflow, Store store, QString stationLabel,
                                       QString storePath, QByteArray keyEncryptionKey,
                                       QWidget *parent)
    : QMainWindow(parent), workflow_(std::move(workflow)), store_(std::move(store)),
      stationLabel_(std::move(stationLabel)), storePath_(std::move(storePath)),
      keyEncryptionKey_(std::move(keyEncryptionKey)) {
    // The one thing that must never be ambiguous on a registration station is
    // whether it is reading a real finger, so the mode is in the title bar.
    const QString sensorText = sensorMode() == SensorMode::Mock
        ? tr("SENSOR: MOCK")
        : (sensorMode() == SensorMode::Real ? tr("SENSOR: real")
                                            : tr("SENSOR: UNBOUND"));
    setWindowTitle(tr("eel BVR — %1 — %2").arg(stationLabel_, sensorText));
    resize(900, 680);

    steps_ = new QStackedWidget(this);
    auto *central = new QWidget(this);
    auto *layout = new QVBoxLayout(central);
    layout->addWidget(steps_, 1);
    setCentralWidget(central);

    progress_ = new QLabel(this);
    statusBar()->addPermanentWidget(progress_);
    statusBar()->showMessage(tr("Station instance %1").arg(store_.instanceId()));

    buildSteps();
    updateProgress();
}

void RegistrationWindow::buildSteps() {
    for (int i = 0; i < workflow_.stepCount(); ++i) {
        switch (workflow_.stepKind(i)) {
        case EEL_STEP_DATA_ENTRY:
            steps_->addWidget(buildDataStep(i));
            break;
        case EEL_STEP_CAPTURE:
            steps_->addWidget(buildCaptureStep(i));
            break;
        case EEL_STEP_REVIEW:
            steps_->addWidget(buildReviewStep());
            break;
        case EEL_STEP_SUBMIT:
            steps_->addWidget(buildSubmitStep());
            break;
        default:
            break;
        }
    }
}

QWidget *RegistrationWindow::buildDataStep(int index) {
    auto *page = new QWidget(steps_);
    auto *outer = new QVBoxLayout(page);

    auto *heading = new QLabel(workflow_.stepLabel(index), page);
    QFont headingFont = heading->font();
    headingFont.setPointSize(headingFont.pointSize() + 4);
    headingFont.setBold(true);
    heading->setFont(headingFont);
    outer->addWidget(heading);

    auto *scroll = new QScrollArea(page);
    scroll->setWidgetResizable(true);
    auto *body = new QWidget(scroll);
    auto *form = new QFormLayout(body);

    const QList<QJsonObject> fields = workflow_.stepFields(index);
    for (const QJsonObject &field : fields) {
        const QString name = field.value(QStringLiteral("name")).toString();
        const QString kind = field.value(QStringLiteral("kind")).toString();
        const bool required = field.value(QStringLiteral("required")).toBool();
        const QString help = field.value(QStringLiteral("help")).toString();

        QWidget *editor = nullptr;
        QString label = field.value(QStringLiteral("label")).toString();
        if (required) {
            label += tr(" *");
        }

        if (kind == QStringLiteral("choice")) {
            auto *combo = new QComboBox(body);
            const QJsonObject constraints = field.value(QStringLiteral("constraints")).toObject();
            for (const QJsonValue &option : constraints.value(QStringLiteral("options")).toArray()) {
                combo->addItem(option.toString());
            }
            editors_.insert(name, combo);
            editor = combo;
        } else if (kind == QStringLiteral("consent")) {
            auto *box = new QCheckBox(body);
            if (!help.isEmpty()) {
                box->setToolTip(help);
            }
            editors_.insert(name, box);
            editor = box;
        } else if (kind == QStringLiteral("date")) {
            auto *date = new QDateEdit(body);
            date->setCalendarPopup(true);
            date->setDisplayFormat(QStringLiteral("yyyy-MM-dd"));
            date->setDate(QDate::currentDate().addYears(-30));
            editors_.insert(name, date);
            editor = date;
        } else {
            auto *line = new QLineEdit(body);
            if (!help.isEmpty()) {
                line->setPlaceholderText(help);
            }
            editors_.insert(name, line);
            editor = line;
        }

        if (!help.isEmpty() && kind != QStringLiteral("consent")) {
            auto *caption = new QLabel(help, body);
            caption->setWordWrap(true);
            caption->setStyleSheet(QStringLiteral("color: palette(mid);"));
            form->addRow(label, editor);
            form->addRow(QString(), caption);
        } else {
            form->addRow(label, editor);
        }
    }

    scroll->setWidget(body);
    outer->addWidget(scroll, 1);
    outer->addLayout(makeNavRow(index));
    return page;
}

QWidget *RegistrationWindow::buildCaptureStep(int index) {
    auto *page = new QWidget(steps_);
    auto *layout = new QVBoxLayout(page);

    const QJsonObject requirement = workflow_.captureRequirement(index);
    const QString label = requirement.value(QStringLiteral("label")).toString();
    const QStringList kinds = captureKindsFor(index);
    const bool optional = requirement.value(QStringLiteral("optional")).toBool();

    auto *heading = new QLabel(workflow_.stepLabel(index), page);
    QFont font = heading->font();
    font.setPointSize(font.pointSize() + 4);
    font.setBold(true);
    heading->setFont(font);
    layout->addWidget(heading);

    auto *body = new QLabel(
        tr("Acceptable acquisitions: %1\n\n"
           "The kit reports which sensors it has. A modality this station cannot "
           "acquire is disabled rather than silently skipped.")
            .arg(kinds.join(QStringLiteral(", "))),
        page);
    body->setWordWrap(true);
    layout->addWidget(body);

    auto *options = new QGroupBox(tr("Captured"), page);
    auto *optionsLayout = new QVBoxLayout(options);

    // Ask the sensor layer what this station can actually acquire, so an
    // unavailable modality is greyed out rather than offered and then failing
    // after the officer has positioned the citizen.
    const QVector<ModalityInfo> available = sensorModalities();
    for (const QString &kind : kinds) {
        auto *box = new QCheckBox(kind, options);
        bool supported = false;
        for (const ModalityInfo &info : available) {
            if (info.name == kind) {
                supported = info.supported;
                break;
            }
        }
        box->setEnabled(supported);
        if (!supported) {
            box->setToolTip(
                sensorMode() == SensorMode::Unbound
                    ? tr("No sensor is bound to this client, so no capture can run.")
                    : tr("This station has no sensor for %1.").arg(kind));
        }
        optionsLayout->addWidget(box);
        captureBoxes_.insert(kind, box);
        captureStepIds_.insert(kind, workflow_.stepLabel(index));
    }

    auto *acquire = new QPushButton(tr("Acquire selected"), page);
    connect(acquire, &QPushButton::clicked, this, [this] { acquireSelected(); });
    optionsLayout->addWidget(acquire);
    layout->addWidget(options);

    if (optional) {
        auto *note = new QLabel(
            tr("This step is optional. Leave everything unticked to continue."), page);
        note->setWordWrap(true);
        note->setStyleSheet(QStringLiteral("color: palette(mid);"));
        layout->addWidget(note);
    }

    layout->addStretch(1);
    layout->addLayout(makeNavRow(index));
    return page;
}

QWidget *RegistrationWindow::buildReviewStep() {
    auto *page = new QWidget(steps_);
    auto *layout = new QVBoxLayout(page);

    auto *heading = new QLabel(tr("Review"), page);
    QFont font = heading->font();
    font.setPointSize(font.pointSize() + 4);
    font.setBold(true);
    heading->setFont(font);
    layout->addWidget(heading);

    auto *summary = new QTextEdit(page);
    summary->setReadOnly(true);
    layout->addWidget(summary, 1);
    review_ = summary;

    layout->addLayout(makeNavRow(-1));
    return page;
}

QWidget *RegistrationWindow::buildSubmitStep() {
    auto *page = new QWidget(steps_);
    auto *layout = new QVBoxLayout(page);

    auto *heading = new QLabel(tr("Submit"), page);
    QFont font = heading->font();
    font.setPointSize(font.pointSize() + 4);
    font.setBold(true);
    heading->setFont(font);
    layout->addWidget(heading);

    auto *body = new QLabel(
        tr("Submitting seals the record into the encrypted offline store and queues "
           "it for sync. Nothing leaves this station until a sync run is started, "
           "so a registration completed offline is never lost."),
        page);
    body->setWordWrap(true);
    layout->addWidget(body);

    auto *submit = new QPushButton(tr("Seal and queue for sync"), page);
    connect(submit, &QPushButton::clicked, this, &RegistrationWindow::onSubmit);
    layout->addWidget(submit);
    layout->addStretch(1);

    status_ = new QLabel(page);
    status_->setWordWrap(true);
    layout->addWidget(status_);

    return page;
}

QHBoxLayout *RegistrationWindow::makeNavRow(int stepIndex) {
    auto *row = new QHBoxLayout;
    auto *back = new QPushButton(tr("Back"), steps_);
    auto *next = new QPushButton(tr("Next"), steps_);

    connect(back, &QPushButton::clicked, this, [this, stepIndex] {
        if (stepIndex > 0) {
            steps_->setCurrentIndex(steps_->currentIndex() - 1);
            updateProgress();
        }
    });
    connect(next, &QPushButton::clicked, this, [this, stepIndex] {
        // A negative index means the page has no meaningful forward validation.
        if (stepIndex < 0) {
            steps_->setCurrentIndex(steps_->currentIndex() + 1);
            updateProgress();
            return;
        }
        if (collect(stepIndex)) {
            steps_->setCurrentIndex(steps_->currentIndex() + 1);
            updateProgress();
        }
    });

    row->addWidget(back);
    row->addStretch(1);
    row->addWidget(next);
    return row;
}

void RegistrationWindow::acquireSelected() {
    for (auto it = captureBoxes_.constBegin(); it != captureBoxes_.constEnd(); ++it) {
        if (!it.value()->isChecked() || !it.value()->isEnabled()) {
            continue;
        }
        const QString kind = it.key();
        const QString method = (kind == QStringLiteral("document_mrz"))
                                   ? QStringLiteral("document_mrz")
                                   : QStringLiteral("biometric_sensor");

        QString error;
        const std::optional<QJsonObject> capture = sensorCapture(kind, 30, &error);
        if (!capture) {
            QMessageBox::critical(this, tr("Capture failed"),
                                  tr("Could not acquire %1: %2").arg(kind, error));
            continue;
        }

        // A capture that failed its presentation-attack check must not be
        // recorded as a successful enrolment, so the driver already refuses it.
        // Record the quality and liveness method for the audit trail.
        if (const QJsonValue verdict = capture->value(QStringLiteral("liveness"));
            verdict.isObject()) {
            statusBar()->showMessage(
                tr("%1 captured; liveness %2 via %3")
                    .arg(kind)
                    .arg(verdict.toObject().value(QStringLiteral("method")).toString())
                    .arg(verdict.toObject().value(QStringLiteral("score")).toInt()));
        } else {
            statusBar()->showMessage(
                tr("%1 captured (%2 bytes)").arg(kind).arg(capture->value(QStringLiteral("bytes")).toInt()));
        }

        const QString stepId = captureStepIds_.value(kind);
        if (const std::optional<QString> id = store_.append(
                kind, QStringLiteral("A"), QStringLiteral("\"%1\"").arg(capture->value(QStringLiteral("sample_id")).toString()),
                operatorId_, workflow_.id(), stepId, QStringLiteral("enrolment"), method, &error)) {
            recordIds_.insert(kind, *id);
        } else {
            QMessageBox::critical(this, tr("Store failure"),
                                  tr("Could not seal %1: %2").arg(kind, error));
        }
    }
    persist();
}

QStringList RegistrationWindow::captureKindsFor(int index) const {
    QStringList kinds;
    for (const QJsonValue &value :
         workflow_.captureRequirement(index).value(QStringLiteral("kinds")).toArray()) {
        kinds << value.toString();
    }
    return kinds;
}

/// Reads the current step's widgets and appends them to the store.
bool RegistrationWindow::collect(int stepIndex) {
    if (workflow_.stepKind(stepIndex) != EEL_STEP_DATA_ENTRY) {
        return true;
    }

    for (const QJsonObject &field : workflow_.stepFields(stepIndex)) {
        const QString name = field.value(QStringLiteral("name")).toString();
        const QString kind = field.value(QStringLiteral("kind")).toString();
        const QString dataClass = field.value(QStringLiteral("data_class")).toString();
        if (!editors_.contains(name)) {
            continue;
        }
        QWidget *editor = editors_.value(name);
        QString wire;

        if (auto *box = qobject_cast<QCheckBox *>(editor)) {
            if (!box->isChecked()) {
                // Absent is a legitimate state for an unticked consent box; the
                // validator decides whether that is allowed.
                continue;
            }
            wire = QStringLiteral("true");
            submission_.insert(name, true);
        } else if (auto *combo = qobject_cast<QComboBox *>(editor)) {
            const QString text = combo->currentText();
            if (text.isEmpty()) {
                continue;
            }
            wire = QStringLiteral("\"%1\"").arg(text);
            submission_.insert(name, text);
        } else if (auto *date = qobject_cast<QDateEdit *>(editor)) {
            const QString text = date->date().toString(QStringLiteral("yyyy-MM-dd"));
            wire = QStringLiteral("\"%1\"").arg(text);
            submission_.insert(name, text);
        } else if (auto *line = qobject_cast<QLineEdit *>(editor)) {
            const QString text = line->text().trimmed();
            if (text.isEmpty()) {
                continue;
            }
            // Encode through QJsonArray so a value containing a quote or
            // backslash cannot produce malformed JSON that the store would
            // reject as a confusing "value is not valid JSON".
            const QByteArray encoded =
                QJsonDocument(QJsonArray{text}).toJson(QJsonDocument::Compact);
            wire = QString::fromUtf8(encoded.mid(1, encoded.size() - 2));
            submission_.insert(name, text);
        }

        if (wire.isEmpty()) {
            continue;
        }
        QString error;
        const QString stepId = workflow_.stepLabel(stepIndex);
        const std::optional<QString> recordId = store_.append(
            name, dataClass, wire, operatorId_, workflow_.id(), stepId,
            QStringLiteral("enrolment"), QStringLiteral("officer_transcription"), &error);
        if (!recordId) {
            QMessageBox::critical(this, tr("Store failure"),
                                  tr("Could not seal %1: %2").arg(name, error));
            return false;
        }
        recordIds_.insert(name, *recordId);
        // Persist immediately rather than at submit: an officer who completes
        // four steps and then loses power must not lose the first three.
        persist();
    }
    return true;
}

void RegistrationWindow::persist() {
    if (storePath_.isEmpty() || keyEncryptionKey_.size() != 32) {
        return;
    }
    QString error;
    if (!store_.sealToDisk(storePath_, keyEncryptionKey_, &error)) {
        QMessageBox::critical(
            this, tr("Store could not be saved"),
            tr("The registration is held in memory but could not be written to "
               "disk. Continuing risks losing it.\n\n%1\n\n%2")
                .arg(storePath_, error));
    }
}

void RegistrationWindow::onSubmit() {
    QStringList captured;
    for (auto it = captureBoxes_.constBegin(); it != captureBoxes_.constEnd(); ++it) {
        if (it.value()->isChecked()) {
            captured << it.key();
        }
    }

    QString error;
    const std::vector<QJsonObject> violations =
        validate(workflow_, submission_, captured, &error);
    if (!violations.empty()) {
        QStringList lines;
        for (const QJsonObject &violation : violations) {
            lines << tr("[%1] %2")
                         .arg(violation.value(QStringLiteral("code")).toString(),
                              violation.value(QStringLiteral("message")).toString());
        }
        QMessageBox::warning(this, tr("Cannot submit"), lines.join(QLatin1Char('\n')));
        return;
    }

    const QJsonObject state = store_.syncState();
    if (status_) {
        status_->setText(tr("Queued. %1 record(s) stored, %2 awaiting sync. "
                            "Station instance %3.")
                             .arg(state.value(QStringLiteral("total")).toInt())
                             .arg(state.value(QStringLiteral("pending")).toInt())
                             .arg(store_.instanceId()));
    }
    persist();
    statusBar()->showMessage(tr("Record sealed. It stays on this station until sync."));
    updateProgress();
}

void RegistrationWindow::updateProgress() {
    if (!progress_) {
        return;
    }
    const int current = steps_->currentIndex() + 1;
    const int total = steps_->count();
    if (review_ && current > 0) {
        QStringList lines;
        for (auto it = recordIds_.constBegin(); it != recordIds_.constEnd(); ++it) {
            lines << tr("%1: sealed").arg(it.key());
        }
        review_->setPlainText(lines.join(QLatin1Char('\n')));
    }
    progress_->setText(tr("Step %1 of %2").arg(current).arg(total));
}

} // namespace eel

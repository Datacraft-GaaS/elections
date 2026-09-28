// SPDX-License-Identifier: Apache-2.0
//
// The BVR registration window. The form is generated from the workflow
// document, so adding a data item is an edit to a document rather than a code
// change.

#pragma once

#include <QLabel>
#include <QMap>
#include <QMainWindow>
#include <QStringList>
#include <QStackedWidget>
#include <QTextEdit>
#include <QWidget>

#include <memory>
#include <optional>

#include "bvr_bridge.h"

class QCheckBox;
class QComboBox;
class QDateEdit;
class QGroupBox;
class QHBoxLayout;
class QLineEdit;
class QVBoxLayout;

namespace eel {

class RegistrationWindow : public QMainWindow {
    Q_OBJECT

public:
    RegistrationWindow(Workflow workflow, Store store, QString stationLabel,
                       QWidget *parent = nullptr);
    ~RegistrationWindow() override = default;

    /// The officer's identity, recorded in every record's provenance.
    void setOperatorId(const QString &operatorId) { operatorId_ = operatorId; }

private slots:
    /// Validates and seals the current registration.
    void onSubmit();

private:
    QWidget *buildDataStep(int index);
    QWidget *buildCaptureStep(int index);
    QWidget *buildReviewStep();
    QWidget *buildSubmitStep();
    QHBoxLayout *makeNavRow(int stepIndex);
    void buildSteps();
    /// Reads a step's widgets, sealing each value into the store.
    bool collect(int stepIndex);
    QStringList captureKindsFor(int index) const;
    void updateProgress();

    Workflow workflow_;
    Store store_;
    QString stationLabel_;
    QString operatorId_ = QStringLiteral("unidentified-officer");

    QStackedWidget *steps_ = nullptr;
    QLabel *progress_ = nullptr;
    QLabel *status_ = nullptr;
    QTextEdit *review_ = nullptr;

    /// Live editors by data-item name, so a step can read its own widgets.
    QMap<QString, QWidget *> editors_;
    /// Capture checkboxes by capture kind.
    QMap<QString, QCheckBox *> captureBoxes_;
    /// Sealed record ids by data-item name.
    QMap<QString, QString> recordIds_;
    /// The values that were sealed, kept for review and submission validation.
    QJsonObject submission_;
};

} // namespace eel

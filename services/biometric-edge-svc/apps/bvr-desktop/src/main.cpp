// SPDX-License-Identifier: Apache-2.0
//
// eel BVR desktop client entry point.
//
// Two jobs: prove the encrypted store is intact before an officer enters any
// citizen data, and compile the workflow document before showing a form. A
// client that will not open a tampered store is the point of the integrity
// chain -- finding out afterwards would be worthless.

#include <QApplication>
#include <QCommandLineOption>
#include <QCommandLineParser>
#include <QFile>
#include <QInputDialog>
#include <QMessageBox>
#include <QTextStream>

#include <cstdio>
#include <utility>

#include "registration_window.h"

namespace {

/// Reads a file, or the embedded default when `path` is empty.
QString readWorkflowDocument(const QString &path) {
    if (!path.isEmpty()) {
        QFile file(path);
        if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
            qFatal("cannot open workflow %s: %s", qPrintable(path),
                   qPrintable(file.errorString()));
        }
        return QString::fromUtf8(file.readAll());
    }
    return QString::fromUtf8(EMBEDDED_WORKFLOW);
}

} // namespace

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    QApplication::setApplicationName(QStringLiteral("eel-bvr-desktop"));
    QApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QApplication::setOrganizationName(QStringLiteral("eel"));

    QCommandLineParser parser;
    parser.setApplicationDescription(
        QStringLiteral("Biometric Voter Registration client (M01). "
                       "The enrolment form is generated from a workflow document."));
    parser.addHelpOption();
    parser.addVersionOption();

    const QCommandLineOption workflowOption(
        {QStringLiteral("w"), QStringLiteral("workflow")},
        QStringLiteral("Path to a workflow document. Defaults to the shipped "
                       "standard enrolment."),
        QStringLiteral("path"));
    const QCommandLineOption stationOption(
        {QStringLiteral("s"), QStringLiteral("station")},
        QStringLiteral("Station label recorded in every record's provenance."),
        QStringLiteral("label"),
        QStringLiteral("Unlabelled station"));
    const QCommandLineOption operatorOption(
        {QStringLiteral("o"), QStringLiteral("operator")},
        QStringLiteral("Officer identity recorded in every record's provenance."),
        QStringLiteral("id"));
    const QCommandLineOption checkOption(
        {QStringLiteral("c"), QStringLiteral("check")},
        QStringLiteral("Compile the workflow, verify the store, then exit. No window. "
                       "Use in CI and as a pre-deployment smoke test."));
    parser.addOption(workflowOption);
    parser.addOption(stationOption);
    parser.addOption(operatorOption);
    parser.addOption(checkOption);
    parser.process(app);

    const bool checkOnly = parser.isSet(checkOption);
    // In check mode there is no window to parent a dialog to, so messages go to
    // stderr and the exit code carries the result.
    auto report = [checkOnly](const QString &title, const QString &body) {
        if (checkOnly) {
            QTextStream(stderr) << title << ": " << body << "\n";
        } else {
            QMessageBox::critical(nullptr, title, body);
        }
    };

    // 1. Compile the workflow before anything else. A malformed document must
    //    not be discovered halfway through a citizen's registration.
    QString error;
    const QString document = readWorkflowDocument(parser.value(workflowOption));
    std::optional<eel::Workflow> workflow = eel::Workflow::compile(document, &error);
    if (!workflow) {
        report(QStringLiteral("Workflow rejected"),
               QStringLiteral("The enrolment workflow could not be compiled:\n\n%1")
                   .arg(error));
        return 2;
    }

    // 2. Create the station store and check its integrity before any data is
    //    entered into it.
    std::optional<eel::Store> store =
        eel::Store::create(parser.value(stationOption), &error);
    if (!store) {
        report(QStringLiteral("Store unavailable"), error);
        return 3;
    }
    if (!store->verify(&error)) {
        report(QStringLiteral("Integrity check failed"),
               QStringLiteral("The local record store failed its integrity check and "
                              "must not be used for registration.\n\n%1")
                   .arg(error));
        return 4;
    }

    if (checkOnly) {
        QTextStream out(stdout);
        out << "workflow: " << workflow->id() << " (" << workflow->stepCount()
            << " steps)\n"
            << "store: ok, instance " << store->instanceId() << "\n"
            << "public key: " << store->publicKeyBase64() << "\n";
        return 0;
    }

    // 3. Officer identity is required, because it is recorded in the provenance
    //    of every datum and is the answer to "who captured this".
    QString operatorId = parser.value(operatorOption);
    if (operatorId.isEmpty()) {
        bool accepted = false;
        operatorId = QInputDialog::getText(
            nullptr, QStringLiteral("Officer identity"),
            QStringLiteral("Officer ID recorded in every record's provenance:"),
            QLineEdit::Normal, QString(), &accepted);
        if (!accepted || operatorId.trimmed().isEmpty()) {
            return 5;
        }
        operatorId = operatorId.trimmed();
    }

    eel::RegistrationWindow window(std::move(*workflow), std::move(*store),
                                   parser.value(stationOption));
    window.setOperatorId(operatorId);
    window.show();

    return app.exec();
}

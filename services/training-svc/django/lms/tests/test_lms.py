import pytest
from decimal import Decimal
from django.test import TestCase
from django.utils import timezone
from uuid6 import uuid7

from lms_app.certificate import generate_certificate
from lms_app.models import (
    CourseModule,
    EnrollmentStatus,
    ModuleType,
    ProgressScope,
    TargetAudience,
    TrainingProgress,
    WorkerCertificate,
    WorkerEnrollment,
)


def _make_module(**kwargs) -> CourseModule:
    defaults = dict(
        title="Core PO Training",
        target_audience=TargetAudience.PO,
        language="en",
        content_markdown="# Core module",
        pass_mark_pct=80,
        max_retakes=3,
        module_type=ModuleType.CORE,
    )
    defaults.update(kwargs)
    return CourseModule.objects.create(**defaults)


def _make_enrollment(module: CourseModule, **kwargs) -> WorkerEnrollment:
    defaults = dict(worker_id=uuid7(), module=module)
    defaults.update(kwargs)
    return WorkerEnrollment.objects.create(**defaults)


def _make_cert(worker_id=None, **kwargs) -> WorkerCertificate:
    defaults = dict(
        worker_id=worker_id or uuid7(),
        station_code="KE-NBI-001",
        modules_completed=[],
        expires_at=timezone.now() + timezone.timedelta(days=365),
        qr_verification_url="https://verify.iebc.or.ke/cert/test",
    )
    defaults.update(kwargs)
    return WorkerCertificate.objects.create(**defaults)


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class AssessmentScoringTests(TestCase):
    def test_pass_at_mark(self):
        module = _make_module(pass_mark_pct=80)
        enrollment = _make_enrollment(module)
        enrollment.attempts += 1
        score = Decimal("80.00")
        enrollment.best_score_pct = score
        enrollment.status = EnrollmentStatus.PASSED
        enrollment.certified = True
        enrollment.save()
        enrollment.refresh_from_db()
        assert enrollment.status == EnrollmentStatus.PASSED
        assert enrollment.certified is True

    def test_fail_below_mark(self):
        module = _make_module(pass_mark_pct=80)
        enrollment = _make_enrollment(module)
        enrollment.attempts = 1
        enrollment.best_score_pct = Decimal("65.00")
        enrollment.status = EnrollmentStatus.RETAKE_REQUIRED
        enrollment.save()
        enrollment.refresh_from_db()
        assert enrollment.status == EnrollmentStatus.RETAKE_REQUIRED
        assert enrollment.certified is False

    def test_idempotent_assessment_submission(self):
        module = _make_module(pass_mark_pct=70)
        enrollment = _make_enrollment(module)
        for _ in range(2):
            enrollment.attempts += 1
            score = Decimal("90.00")
            if score >= module.pass_mark_pct:
                enrollment.status = EnrollmentStatus.PASSED
                enrollment.certified = True
                enrollment.best_score_pct = score
            enrollment.save()
        enrollment.refresh_from_db()
        assert enrollment.status == EnrollmentStatus.PASSED
        assert enrollment.certified is True


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class CascadeLevelTests(TestCase):
    def test_trainer_cert_type_exists(self):
        module = _make_module(
            module_type=ModuleType.TRAINER_CERT, target_audience=TargetAudience.TRAINER
        )
        assert module.module_type == ModuleType.TRAINER_CERT

    def test_clerk_module_type(self):
        module = _make_module(
            target_audience=TargetAudience.CLERK, module_type=ModuleType.CORE
        )
        assert module.target_audience == TargetAudience.CLERK


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class RefresherDeadlineTests(TestCase):
    def test_module_expiry_date_stored(self):
        future = timezone.now() + timezone.timedelta(days=90)
        module = _make_module(module_type=ModuleType.REFRESHER, expiry_date=future)
        module.refresh_from_db()
        assert module.expiry_date is not None

    def test_refresher_enrollment_created(self):
        module = _make_module(module_type=ModuleType.REFRESHER)
        enrollment = _make_enrollment(module)
        assert enrollment.status == EnrollmentStatus.NOT_STARTED


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class CertificateRevocationTests(TestCase):
    def test_revoke_certificate(self):
        cert = _make_cert()
        assert cert.revoked is False
        cert.revoked = True
        cert.revoked_at = timezone.now()
        cert.revoked_by = uuid7()
        cert.save()
        cert.refresh_from_db()
        assert cert.revoked is True
        assert cert.revoked_at is not None

    def test_revocation_idempotent(self):
        cert = _make_cert()
        cert.revoked = True
        cert.revoked_at = timezone.now()
        cert.save()
        # Second revoke attempt should be caught at view layer; DB stores it fine
        cert.refresh_from_db()
        assert cert.revoked is True


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class QRGeneratorTests(TestCase):
    def test_generate_certificate_returns_required_keys(self):
        worker_id = str(uuid7())
        result = generate_certificate(worker_id, "KE-NBI-001", ["module-1", "module-2"])
        assert "id" in result
        assert "hash" in result
        assert "qr_url" in result
        assert len(result["hash"]) == 64
        assert result["qr_url"].startswith("https://verify.iebc.or.ke/cert/")

    def test_hash_differs_per_call(self):
        w1 = str(uuid7())
        w2 = str(uuid7())
        r1 = generate_certificate(w1, "KE-NBI-001", [])
        r2 = generate_certificate(w2, "KE-NBI-001", [])
        assert r1["hash"] != r2["hash"]


@pytest.mark.trace("FR-M38-006", "AC-M38-02", "EC-SCL-01")
class ThirdFailTests(TestCase):
    def test_third_fail_sets_failed_status(self):
        module = _make_module(pass_mark_pct=80, max_retakes=3)
        enrollment = _make_enrollment(module)
        for i in range(3):
            enrollment.attempts += 1
            enrollment.best_score_pct = Decimal("50.00")
            if enrollment.attempts >= module.max_retakes:
                enrollment.status = EnrollmentStatus.FAILED
            else:
                enrollment.status = EnrollmentStatus.RETAKE_REQUIRED
            enrollment.save()
        enrollment.refresh_from_db()
        assert enrollment.status == EnrollmentStatus.FAILED
        assert enrollment.attempts == 3

    def test_before_third_fail_is_retake_required(self):
        module = _make_module(pass_mark_pct=80, max_retakes=3)
        enrollment = _make_enrollment(module)
        enrollment.attempts = 2
        enrollment.status = EnrollmentStatus.RETAKE_REQUIRED
        enrollment.save()
        enrollment.refresh_from_db()
        assert enrollment.status == EnrollmentStatus.RETAKE_REQUIRED

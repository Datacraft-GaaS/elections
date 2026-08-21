"""Tests for adjudication models (no database required for unmanaged models)."""

import uuid
from datetime import timedelta
from unittest.mock import MagicMock

import pytest
from django.utils import timezone


def make_case(
    priority="Standard",
    tier="Tier1Analyst",
    state="Open",
    sla_offset_hours=4,
):
    """Return a mock AdjudicationCase without hitting the database."""
    from abis_console.apps.adjudication.models import AdjudicationCase

    case = AdjudicationCase.__new__(AdjudicationCase)
    case.id = uuid.uuid4()
    case.opened_at = timezone.now()
    case.priority = priority
    case.tier = tier
    case.state = state
    case.enrolment_job_id = uuid.uuid4()
    case.probable_match_subject_ids = [str(uuid.uuid4())]
    case.evidence_bundle_ref = uuid.uuid4()
    case.assigned_to = None
    case.assigned_at = None
    case.decided_at = None
    case.decision_type = None
    case.decision_subject_id = None
    case.decision_reason = None
    case.sla_deadline_at = timezone.now() + timedelta(hours=sla_offset_hours)
    case.created_at = timezone.now()
    case.updated_at = timezone.now()
    return case


@pytest.mark.django_db(transaction=False)
class TestAdjudicationCaseModel:
    def test_str_representation(self):
        case = make_case()
        result = str(case)
        assert "Open" in result

    def test_sla_hours_remaining_positive_when_not_breached(self):
        case = make_case(sla_offset_hours=4)
        assert case.sla_hours_remaining() > 0

    def test_sla_hours_remaining_negative_when_breached(self):
        case = make_case(sla_offset_hours=-1)
        assert case.sla_hours_remaining() < 0

    def test_is_sla_breached_true_when_past_deadline(self):
        case = make_case(sla_offset_hours=-2)
        assert case.is_sla_breached() is True

    def test_is_sla_breached_false_when_within_deadline(self):
        case = make_case(sla_offset_hours=3)
        assert case.is_sla_breached() is False

    def test_is_decided_true_for_decided_state(self):
        case = make_case(state="Decided")
        assert case.is_decided() is True

    def test_is_decided_true_for_withdrawn_state(self):
        case = make_case(state="Withdrawn")
        assert case.is_decided() is True

    def test_is_decided_false_for_open_state(self):
        case = make_case(state="Open")
        assert case.is_decided() is False

    def test_election_day_hold_priority_string(self):
        case = make_case(priority="ElectionDayHold")
        assert case.priority == "ElectionDayHold"

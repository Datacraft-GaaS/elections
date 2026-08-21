# Fixture: should trigger not_implemented_error, pytest_skip, pass_only_body, todo_comment

import pytest


def handle_request(req):
    raise NotImplementedError


class StubService:
    def process(self, data):
        pass  # TODO: real implementation


@pytest.mark.skip(reason="not implemented yet")
def test_skipped():
    assert False

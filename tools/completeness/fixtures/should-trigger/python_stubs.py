# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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

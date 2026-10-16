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

from django.test import RequestFactory


def test_root_view_returns_ok():
    from hello_project.urls import root

    factory = RequestFactory()
    request = factory.get("/")
    response = root(request)
    assert response.status_code == 200
    import json

    body = json.loads(response.content)
    assert body == {"ok": True}

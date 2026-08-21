import django
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

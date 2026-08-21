from django.http import JsonResponse
from django.urls import path


def root(request):  # type: ignore[no-untyped-def]
    return JsonResponse({"ok": True})


urlpatterns = [
    path("", root),
]

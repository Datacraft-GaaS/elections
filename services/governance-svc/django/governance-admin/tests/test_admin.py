from django.test import TestCase


class AdminSmokeTest(TestCase):
    def test_admin_login_page(self):
        resp = self.client.get("/admin/login/")
        self.assertEqual(resp.status_code, 200)

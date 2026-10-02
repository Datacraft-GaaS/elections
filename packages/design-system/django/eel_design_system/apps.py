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

from django.apps import AppConfig


class EelDesignSystemConfig(AppConfig):
    name = "eel_design_system"
    label = "eel_design_system"
    verbose_name = "eel Design System"

    # No import-time hooks required; template tags are discovered by Django
    # automatically because the package ships an ``templatetags`` module.
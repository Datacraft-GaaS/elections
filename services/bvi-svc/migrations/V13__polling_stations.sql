-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- Polling station registry mirror used by getVoterStation. Populated by the
-- boundary sync job from boundary-svc boundaries (kind = 'polling_station');
-- this service never invents station metadata on its own.
CREATE TABLE IF NOT EXISTS polling_stations (
  station_code      TEXT PRIMARY KEY,
  name              TEXT NOT NULL,
  constituency_code TEXT NOT NULL,
  county_code       TEXT NOT NULL
);

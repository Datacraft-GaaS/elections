# Synthetic pgBackRest base backup fixture

This directory contains stub files for CI and dry-run exercises.
It is not a real pgBackRest archive. Scripts operating in `EEL_DR_MODE=dry` validate
that this directory exists and is non-empty; no pgBackRest operations are run against it.

Real drills use the live pgBackRest stanza `eel` backed by pre-prod MinIO.

## Contents

- `backup.info` -- stub stanza info file
- `backup/` -- empty directory simulating the backup label structure

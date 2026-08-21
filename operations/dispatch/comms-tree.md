# Comms tree

## Primary escalation chain

```
Officer
  -> Presiding Officer (station level)
    -> County Returning Officer
      -> National Field Operations Lead
        -> Incident Commander
          -> Election Commission Chair
```

## Contact methods by channel

| Role | Primary | Backup (SMS) | Backup (USSD) | Backup (WhatsApp) |
|------|---------|-------------|---------------|-------------------|
| Officer | eel field app | +254-XXX-XXXXXX | *XXX# | +254-XXX-XXXXXX |
| Presiding Officer | eel field app | +254-XXX-XXXXXX | *XXX# | +254-XXX-XXXXXX |
| County Returning Officer | ECC console | +254-XXX-XXXXXX | N/A | +254-XXX-XXXXXX |
| National Field Ops Lead | ECC console + PagerDuty | +254-XXX-XXXXXX | N/A | +254-XXX-XXXXXX |
| Incident Commander | War room bridge | +254-XXX-XXXXXX | N/A | Secure channel |
| SRE on-call | PagerDuty | +254-XXX-XXXXXX | N/A | N/A |

Contact numbers are stored in `support-admin-svc` and are not embedded in this document for security reasons. Retrieve current contacts from the ECC console under "Contacts".

## Escalation triggers

| Situation | Escalate to | Channel |
|-----------|------------|---------|
| Officer missing at assigned station | Presiding Officer | eel app or SMS |
| Kit failure preventing check-in | Presiding Officer + Field Support | eel app + helpdesk |
| GPS spoof suspected | National Field Ops Lead | ECC console |
| Station access denied | County Returning Officer | Phone |
| Civil disturbance at station | County Returning Officer + law enforcement | Phone |
| Tally dispute | County Returning Officer + Legal | ECC console + phone |
| System outage affecting station | SRE on-call | PagerDuty |
| P0 platform incident | Incident Commander | War room bridge + PagerDuty |

## Backup channel procedures

### SMS fallback

If the eel field app is unavailable, officers send a structured SMS to the field ops gateway number:

```
CHECK-IN <STATION_ID> <OFFICER_ID> <STATUS>
```

The gateway parses the SMS and injects a synthetic check-in event into `field-mobile-svc`.

### USSD fallback

Officers dial `*XXX#` to access the USSD check-in menu. USSD responses are queued and flushed to `field-mobile-svc` once connectivity is restored.

### WhatsApp fallback

A monitored WhatsApp number is available for presiding officers and above. Officers should use SMS USSD as the first fallback; WhatsApp is a second fallback.

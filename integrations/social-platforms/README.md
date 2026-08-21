# integrations/social-platforms

Cross-platform social media content verification and audience reach analytics.

**MoU Status: Platform agreements pending (Facebook, Twitter, LinkedIn partnership agreements under review)**

## Module usage

- M72 (civic-broadcast) - verify social media reach for civic announcements
- M88 (disinformation-detect) - verify authentic content sources and account metadata
- M91 (election-comms) - reach analytics for candidate communications

## Wire protocol and authentication

Protocol: REST (HTTPS)
Authentication: OAuth2 per platform (separate OAuth tokens for Facebook, Twitter, LinkedIn)
Endpoints: Facebook Graph API, Twitter v2 API, LinkedIn API

### Endpoints

| Platform | Path | Method | Description |
|---|---|---|---|
| Facebook | `/v18.0/me/posts` | GET | List authenticated user posts |
| Facebook | `/v18.0/{id}/insights` | GET | Get post reach and engagement metrics |
| Twitter | `/2/tweets/search/recent` | GET | Search recent tweets |
| Twitter | `/2/users/{id}/tweets` | GET | Get user timeline |
| LinkedIn | `/v2/me` | GET | Get authenticated user profile |
| LinkedIn | `/v2/organizationalEntities/{id}/shares` | GET | Get organization shares |

## PII classification (ADR-0016)

| Field | Tier | Notes |
|---|---|---|
| userName | B (Internal) | Social media handle |
| userProfile | B (Internal) | Public profile info (with auth context) |
| postContent | B (Internal) | User-generated content |
| engagementMetrics | C (Public) | Aggregated reach/like counts |
| accountVerified | C (Public) | Account verification status |

## Request and response shape

**Content Query Request:**
- platform: enum (facebook, twitter, linkedin, required)
- query: string (required, search term)
- accountID: string (required for user timeline queries)
- timeRange: object {startDate: ISO 8601, endDate: ISO 8601}

**Content Query Response:**
- platform: string
- results: array [{contentID, author, text, postedDate, likes, shares, reach}]
- paginationToken: string (for next page)

## Sample usage

```bash
# Query recent tweets (requires Twitter OAuth2 token)
curl -X GET "https://api.twitter.com/2/tweets/search/recent?query=election%202025" \
  -H "Authorization: Bearer $TWITTER_TOKEN"

# Get Facebook post insights (requires Facebook OAuth2 token)
curl -X GET "https://graph.facebook.com/v18.0/POST_ID/insights" \
  -H "Authorization: Bearer $FACEBOOK_TOKEN"

# Get LinkedIn organization shares (requires LinkedIn OAuth2 token)
curl -X GET "https://api.linkedin.com/v2/organizationalEntities/ID/shares" \
  -H "Authorization: Bearer $LINKEDIN_TOKEN"
```

## Health and readiness

```bash
# Health check per platform
curl -s https://graph.facebook.com/me?access_token=$FACEBOOK_TOKEN
curl -s -H "Authorization: Bearer $TWITTER_TOKEN" https://api.twitter.com/2/tweets/search/recent?query=test
curl -s -H "Authorization: Bearer $LINKEDIN_TOKEN" https://api.linkedin.com/v2/me
```

## Rate limits

Limits vary by platform and OAuth scope

| Platform | Metric | Limit | Window |
|---|---|---|---|
| Facebook | API calls | 200/hour | rolling hour |
| Twitter | Tweet search | 450/15min | rolling 15-minute |
| Twitter | User timeline | 300/15min | rolling 15-minute |
| LinkedIn | API calls | 300/min | rolling 1-minute |

All platforms subject to rate limit headers in responses.

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors (5xx, 429, timeout, connection refused) |
| open duration | 30 seconds |
| retry policy | exponential backoff (base 100ms, max 5s) |

Per-platform independent circuit breakers.

## Chaos testing

Set `SOCIAL_CHAOS` to enable failure injection in dev/staging:

| Mode | Effect |
|---|---|
| timeout | return 504 after platform-specific timeout |
| error | return 429 Rate Limit Exceeded |
| slow | add 3s latency to all requests |
| invalid_token | return 401 Unauthorized |

## Fallback strategy

Circuit open per platform: queue content verification request. Fallback to cached last-known account verification status (TTL: 24 hours). For reach analytics, use historical averages from past 7 days. Queue TTL: 12 hours.

## Environment variables

| Variable | Required | Default | Purpose |
|---|---|---|---|
| FACEBOOK_CLIENT_ID | yes | - | Facebook OAuth2 client ID |
| FACEBOOK_CLIENT_SECRET | yes | - | Facebook OAuth2 client secret |
| FACEBOOK_TOKEN | yes | - | Facebook access token |
| TWITTER_CLIENT_ID | yes | - | Twitter OAuth2 client ID |
| TWITTER_CLIENT_SECRET | yes | - | Twitter OAuth2 client secret |
| TWITTER_TOKEN | yes | - | Twitter Bearer token |
| LINKEDIN_CLIENT_ID | yes | - | LinkedIn OAuth2 client ID |
| LINKEDIN_CLIENT_SECRET | yes | - | LinkedIn OAuth2 client secret |
| LINKEDIN_TOKEN | yes | - | LinkedIn access token |
| SOCIAL_CHAOS | no | false | Chaos mode (timeout/error/slow) |
| SOCIAL_TIMEOUT_SECS | no | 15 | Request timeout per platform |

## Related modules

- M72 (civic-broadcast) - primary consumer for reach analytics
- M88 (disinformation-detect) - validates authentic sources
- M91 (election-comms) - tracks candidate communication spread

## Runbook

Refer to `docs/runbooks/integrations/social-platforms.md` for OAuth2 token refresh workflows and per-platform rate limit handling.

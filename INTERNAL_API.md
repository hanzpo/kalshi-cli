# Kalshi Internal API Notes

These endpoints are undocumented and not part of the official trade API spec.
They may change without notice.

## Semantic Search

**`GET https://api.elections.kalshi.com/v1/search/series`**

Server-side fuzzy/semantic search for series. Returns series with nested events and markets.
This is a v1 endpoint (not under `/trade-api/v2`).

### Query Parameters

| Param               | Type            | Notes                                                  |
|---------------------|-----------------|--------------------------------------------------------|
| query               | string          | Search query (max 1000 chars)                          |
| order_by            | enum            | VOLUME, NEWEST, TRENDING, CLOSING, QUERYMATCH          |
| page_size           | int32           | Allowed: 3, 5, 8, 25, 30, 50, 70, 100                 |
| reverse             | bool            | Reverse sort order                                     |
| cursor              | object          | Pagination cursor (score, series_ticker, event_ticker) |
| statuses            | repeated string | Filter by market status                                |
| frequencies         | repeated string | Filter by frequency                                    |
| categories          | repeated string | Filter by category                                     |
| tags                | repeated string | Filter by tag                                          |
| excluded_categories | repeated string | Exclude categories                                     |
| excluded_tags       | repeated string | Exclude tags                                           |
| scopes              | repeated string | Filter by scope                                        |
| competitions        | repeated string | Filter by competition                                  |
| competition_scopes  | repeated string | Competition scopes                                     |
| fee_types           | repeated string | Filter by fee type                                     |
| keywords            | repeated string | Filter by keywords                                     |
| with_milestones     | bool            | Include milestones                                     |
| embedding_search    | bool            | Use embedding-based search                             |
| force_elasticsearch | bool            | Force Elasticsearch                                    |
| fuzzy_threshold     | int             | Fuzzy matching threshold                               |

### Example

```
GET https://api.elections.kalshi.com/v1/search/series?order_by=querymatch&query=duke&page_size=10&fuzzy_threshold=4
```

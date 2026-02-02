# Skedda API Reference

Reference documentation for the Skedda booking platform API. All endpoints use cookie-based session auth and require a CSRF token.

## Hierarchy

The hierarchy is:

```
  Venue 72329 ("Switchyards Atlanta")
    └── Space Tags (locations)
          ├── "Buckhead"        → spaceIds: [1077113, 1077114, ...]
          ├── "Decatur"         → spaceIds: [1077096, 1077097,  ...]
          ├── "Avondale Estates" → spaceIds: [1423126, 1423127, ...]
          ├── "Downtown"        → spaceIds: [1025335, 1025334, ...]
          └── ... (13 locations total)
                └── Spaces (individual rooms/booths)
```

## Base URL

```
https://{subdomain}.skedda.com
```

For Switchyards, the subdomain is `switchyards`.

---

## Authentication

### 1. Fetch Login Page (CSRF)

Before logging in, you must fetch the login page to obtain the CSRF cookie and token. The old `www.skedda.com/logins` endpoint no longer accepts POST requests (returns 405). All auth now goes through `app.skedda.com`.

```
GET https://app.skedda.com/account/login
```

**Response:**
- Sets cookie: `X-Skedda-RequestVerificationCookie` (domain `.skedda.com`)
- HTML body contains: `<input name="__RequestVerificationToken" ... value="{TOKEN}" />`

Both the cookie and the token value are required for the login POST.

### 2. Login

```
POST https://app.skedda.com/logins
Content-Type: application/json
X-Skedda-RequestVerificationToken: {TOKEN from step 1}
```

**Request body:**

```json
{
  "login": {
    "username": "user@example.com",
    "password": "password",
    "rememberMe": false,
    "arbitraryerrors": null
  }
}
```

**Response:** `200 OK` on success. Sets `X-Skedda-ApplicationCookie` (domain `.skedda.com`), which is the session cookie used for all subsequent API calls across subdomains.

### 3. Discover Primary Domain (subdomain)

After login, discover which subdomain the user belongs to.

```
POST https://app.skedda.com/account/login
Content-Type: application/x-www-form-urlencoded
```

**Request body:** `username=user@example.com`

**Response:** `302` redirect to `https://{subdomain}.skedda.com/...`. Extract the subdomain from the redirect `Location` header. Do **not** follow the redirect automatically.

---

## CSRF Token

Required for all authenticated API calls on the subdomain. Sent as the `X-Skedda-RequestVerificationToken` header.

```
GET https://{subdomain}.skedda.com/booking
```

Parse the HTML response for:

```html
<input name="__RequestVerificationToken" ... value="{TOKEN}" />
```

The token value is used in subsequent requests. Note: this is a *different* token from the one used during login — each page issues its own CSRF token.

---

## Endpoints

All endpoints below require:
- Session cookies from authentication
- Header: `X-Skedda-RequestVerificationToken: {token}`

### GET /webs — Venue & Spaces

Fetches venue details and all bookable spaces.

```
GET https://{subdomain}.skedda.com/webs
```

**Response shape:**

```json
{
  "venue": [
    {
      "id": "72329",
      "name": "Venue Name",
      "spacePresentation": {
        "spaceTags": [
          {
            "name": "Location Name",
            "spaceIds": [1, 2, 3]
          }
        ]
      }
    }
  ],
  "spaces": [
    {
      "id": 123,
      "name": "Room A"
    }
  ]
}
```

- `venue` is an array (typically one element).
- `spaces` contains all bookable leaf spaces. Items with a `spaceIds` field are parent/group objects and should be skipped when listing bookable rooms.
- `venue[0].spacePresentation.spaceTags` maps location names to their space IDs.

### POST /webs — Other Subdomains

Fetches other subdomains associated with the account.

```
POST https://{subdomain}.skedda.com/webs
```

**Response shape:**

```json
{
  "web": {
    "otherSubdomains": {
      "subdomain1": "Label 1",
      "subdomain2": "Label 2"
    }
  }
}
```

### GET /bookingslists — Existing Bookings

Fetches all bookings within a time range. **This is how you determine availability** — there is no dedicated availability endpoint.

```
GET https://{subdomain}.skedda.com/bookingslists?start={start}&end={end}
```

**Query parameters:**
- `start` — URL-encoded datetime string: `2025-01-15T00:00:00`
- `end` — URL-encoded datetime string: `2025-01-15T23:59:59`

**Datetime format:** `YYYY-MM-DDTHH:MM:SS` (no timezone suffix)

**Response shape:**

```json
{
  "bookings": [
    {
      "start": "2025-01-15T09:00:00",
      "end": "2025-01-15T10:00:00",
      "title": "Meeting",
      "spaces": [123],
      "venue": 72329
    }
  ]
}
```

**Important:** Recurring bookings are returned even if they fall outside the requested time window. You must manually filter them by checking whether their time-of-day overlaps with the query range.

### POST /bookings — Create a Booking

Creates a new booking for one or more spaces.

```
POST https://{subdomain}.skedda.com/bookings
Content-Type: application/json
```

**Request body:**

```json
{
  "booking": {
    "start": "2025-01-15T09:00:00",
    "end": "2025-01-15T10:00:00",
    "title": "My Booking",
    "venue": 72329,
    "spaces": [123, 456],
    "type": 1,
    "price": 0
  }
}
```

- `venue` — integer venue ID (from `/webs`)
- `spaces` — array of integer space IDs
- `start`/`end` — truncated to the minute
- `type` — `1` for standard booking
- `price` — `0` for included/free bookings

**Response:** `200 OK` on success.

---

## Error Responses

Error responses follow this shape:

```json
{
  "errors": [
    {
      "detail": "Human-readable error message"
    }
  ]
}
```

---

## Determining Availability

Skedda does **not** have a dedicated availability endpoint. To determine available time slots:

1. Fetch existing bookings for the desired date range via `GET /bookingslists`
2. Filter bookings to only those for the target space ID(s)
3. Filter out recurring bookings that don't overlap the query time range
4. Calculate free slots by finding gaps between booked periods within your desired time window (e.g., venue operating hours)

---

## Authentication Flow Summary

```
1. GET  app.skedda.com/account/login  → get CSRF cookie + token
2. POST app.skedda.com/logins         → authenticate (with CSRF header), get session cookie
3. GET  {subdomain}.skedda.com/booking → get subdomain CSRF token from HTML
4. GET  /webs                          → get venue + spaces
5. GET  /bookingslists                 → get existing bookings (to derive availability)
6. POST /bookings                      → create a booking
```

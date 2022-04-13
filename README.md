# multibooru-mirror

## Overall design concept
- Separate downloaders for each site, with their own database schema
- Single database is populated from each site using site-specific algorithms
- When a new site is added, its features are added to the common database and not to the site-specific databases
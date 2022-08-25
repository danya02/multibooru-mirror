Format
======

.. toctree::
    :maxdepth: 2

    danbooru


Each site has a somewhat different set of data that it exposes.
Because of this, it is difficult to find a single database schema that would fit all of them.
Instead, the source of truth is a set of JSON records, and those are later processed into database tables offline.

For every successful scrape, a record is created. The record contains all data that has been scraped,
even if the same data was recorded previously.

Each record has the same top-level keys: `v`, `t`, `s`, `k`.

The `v` key contains a string representing the global format being used.
The current version is `"1"`.

The `t` key contains a number that is the Unix time when the record was created.
This corresponds to the time that the corresponding fact was scraped.

The `s` key contains a string that is the identifier of the site from which the data was obtained.

The `t` key contains a string that identifies the type of entity that the record represents.

The `d` key contains an object that contains the data that was scraped.
This must be interpreted with an appropriate processor based on the `s` and `t` keys.

The object inside the `d` key must contain a `v` key, which is a string representing the version of the processor to be used.
This is distinct from the global version -- a site might change its format without the global format changing.

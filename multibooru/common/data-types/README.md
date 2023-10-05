# data-types

This crate provides the data types and entity definitions for things that can be downloaded from imageboards,
such as posts, comments, users, wikis, etc.

It also provides a top-level struct, the `Record`, which is what gets persisted into the database.
It is the top of the hierarchy, containing every kind of message that you might need.
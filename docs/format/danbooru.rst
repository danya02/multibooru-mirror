Site: Danbooru
==============

URL: https://danbooru.donmai.us/

Post
----

.. warning::
    This will change in the future. Do not use version 1 while this warning is here.

```
{
    "v": "1",

    // Post ID
    "id": 123,
    // Post's uploader user
    "uploader_id": 123,
    // Post's approver user
    "approver_id": 123,
    // Post's list of tags
    // Tags have types: general, artist, copyright, character, meta. If there are any left over, they'll be called "none".
    "tags": {
        "general": [
            "tag1",
            "tag2",
            "tag3"
        ],
        "artist": [
            "tag4",
        ]
    },

    // Post's rating
    // Possible values: "g", "s", "q", "e"
    "rating": "g",

    // Post's parent post ID (may be null)
    "parent_id": null,

    // Post's source
    "source": "https://example.com/image.jpg",

    // Post's image MD5 hash
    "md5": "1234567890abcdef",

    // Original file URL
    "file_url": "https://danbooru.donmai.us/data/1234567890abcdef.jpg",
    // Large preview URL
    "large_file_url": "https://danbooru.donmai.us/data/preview/1234567890abcdef.jpg",
    // Small preview URL
    "small_file_url": "https://danbooru.donmai.us/data/preview/small_1234567890abcdef.jpg",
    // File extension for the original file
    "file_ext": "jpg",
    // Original file's size, in bytes
    "file_size": 12345,
    // Image width, in pixels
    "width": 1920,

    // Post's score
    "score": 5,
    // Number of times the post has been favorited
    "fav_count": 17,

    // Last time the post has had comments updated (??), as Unix timestamp. Null if no comments.
    "last_comment_bumped_at": 1234567890,
    // Last time the post has had posts updated (??), as Unix timestamp. Null if no notes.
    "last_noted_at": null,

    // Does the post have children posts?
    "has_children": false,

    // Post's creation time, as Unix timestamp.
    "created_at": 1234567890,
    // Post's update time, as Unix timestamp.
    "updated_at": 1234567890
}
```

from .common import db
from .imageboard import Imageboard
from .post import Post, MissingPost
from .revision import PostRevision
from .tag import Tag, RevisionTag
from .image import Image, DownloadedImage, FileType

db.connect()
db.create_tables([
    Imageboard,
    Post,
    MissingPost,
    PostRevision,
    Tag,
    RevisionTag,
    Image,
    DownloadedImage,
    FileType,
    ])
db.close()



from .common import *
from .revision import PostRevisionMultidata
import peewee as pw

class Tag(MyModel):
    '''A tag is a textual label associated with a post and describing its contents.'''
    id = BigPK()
    name = pw.CharField(unique=True)

class RevisionTag(PostRevisionMultidata):
    '''This is a many-to-many mapping between tags and revisions.'''
    tag = pw.ForeignKeyField(Tag)


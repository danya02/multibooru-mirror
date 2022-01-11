from .common import *
from .post import Post
import peewee as pw
import datetime as dt

class PostRevision(MyModel):
    '''A post revision is a state of a post at a particular point in time.'''
    id = BigPK()
    post = pw.ForeignKeyField(Post)
    recorded_at = pw.DateTimeField(index=True, default=dt.datetime.now, verbose_name='Revision recorded at', help_text='At this time, a new set of properties for this post was recorded. If a property is not associated with this revision, but was associated with a previous revision, it is assumed that it has not changed between then and now.')

class PostRevisionProperty(MyModel):
    '''A post revision property is a single datum that might change between post revisions.'''
    revision = pw.ForeignKeyField(PostRevision, primary_key=True)

class PostRevisionMultidata(MyModel):
    '''A post revision multidata is a list of records associated with a particular post revision.'''
    id = BigPK()
    revision = pw.ForeignKeyField(PostRevision, index=True)


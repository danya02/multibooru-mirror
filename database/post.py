from .common import *
from .imageboard import Imageboard
import peewee as pw
import datetime as dt

class Post(MyModel):
    '''A post is a single "thing" posted on an imageboard.'''
    id = BigPK()
    imageboard = pw.ForeignKeyField(Imageboard)
    local_id = pw.BigIntegerField(index=True, verbose_name='ID in imageboard', help_text='ID that the source imageboard has assigned to this post')
    first_detected_at = pw.DateTimeField(default=dt.datetime.now, verbose_name='First detected date', help_text='Timestamp when this post was first found and recorded')
   
    @property
    def latest_revision(self):
        '''Query that selects the latest revision for this post.'''
        # importing here to avoid circular dependency; for discussion see https://stackoverflow.com/q/3095071/5936187
        from .revision import Revision
        return Revision.select().where(Revision.post == self).order_by(Revision.recorded_at.desc()).limit(1)

    class Meta:
        indexes = (
                ( ('imageboard', 'local_id'), True),
                )

class PostProperty(MyModel):
    '''A post property is a single immutable datum that is associated with a post.'''
    post = pw.ForeignKeyField(Post, primary_key=True)

class MissingPost(MyModel):
    '''
    A missing post is a post whose ID was expected to be found on the imageboard
    (for example because it is in an auto-incrementing sequence, or it was found before),
    but which did not exist at time of scraping. It is not expected to reappear.

    A post with the same ID could still exist, if it was recorded before it became missing.
    '''
    id = BigPK()
    imageboard = pw.ForeignKeyField(Imageboard)
    local_id = pw.BigIntegerField(index=True, verbose_name='ID in imageboard', help_text='ID that this post could have had in the source imageboard')
    first_detected_at = pw.DateTimeField(default=dt.datetime.now, verbose_name='First detected date', help_text='Timestamp when this post was first discovered to be missing')

    class Meta:
        indexes = (
                ( ('imageboard', 'local_id'), True),
                )

import datetime
import peewee as pw

db = pw.SqliteDatabase('danbooru.db')

class MyModel(pw.Model):
    class Meta:
        database = db

# If a record that was asked for was last updated more than this long ago,
# download it again.
STALE_THRESHOLD = datetime.timedelta(days=7)

def create_table(cls):
    db.create_tables([cls])
    return cls

@create_table
class Tag(MyModel):
    id = pw.IntegerField(primary_key=True)
    name = pw.CharField(unique=True)
    category = pw.IntegerField()
    CATEGORIES = {0: 'general', 1: 'artist', 3: 'copyright', 4: 'character', 5: 'meta'}
    post_count = pw.IntegerField()
    is_locked = pw.BooleanField()
    is_deprecated = pw.BooleanField()
    created_at = pw.DateTimeField()
    updated_at = pw.DateTimeField()

    record_updated_at = pw.DateTimeField(index=True)
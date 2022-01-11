import peewee as pw

db = pw.PostgresqlDatabase('multibooru', host='localhost', user='multibooru', password='multibooru')

class MyModel(pw.Model):
    class Meta:
        database = db

class SmallIdentityField(pw.Field):
    field_type = 'smallint generated always as identity'

class BigIdentityField(pw.Field):
    field_type = 'bigint generated always as identity'

def SmallPK(*args, **kwargs):
    return SmallIdentityField(*args, primary_key=True, **kwargs)

def BigPK(*args, **kwargs):
    return BigIdentityField(*args, primary_key=True, **kwargs)

class SHA256Field(pw.Field):
    field_type='bytea'

    def db_value(self, value):
        return bytes.fromhex(value)

    def python_value(self, value):
        return value.hex()

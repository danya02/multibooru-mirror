from .common import *
import peewee as pw

class Imageboard(MyModel):
    '''An imageboard is a single source of posts.'''
    id = SmallPK()
    name = pw.CharField(unique=True, verbose_name='Imageboard name', help_text='Brand name for the imageboard (i.e. Latest images from <name>)')
    url = pw.CharField(unique=True, verbose_name='Homepage URL', help_text='URL to the imageboard\'s index page')


from .common import *
from .post import PostProperty
import peewee as pw

class FileType(MyModel):
    '''A file type is a specific encoding for the file's content, specified by its extension.'''
    id = SmallPK()
    mimetype = pw.CharField(unique=True, verbose_name='MIME type', help_text='RFC2046-style media format description (i.e. "application/octet-stream')
    extension = pw.CharField(unique=True, verbose_name='File extension', help_text='The extension for files of this type as found in the image store.')

class DownloadedImage(MyModel):
    '''
    A downloaded image is a file that was downloaded from an imageboard and linked to a post.

    This model does not refer to Image; instead, Image refers to it.
    This is because the same file could be in multiple imageboards
    but an Image is tied to a specific one.
    '''
    id = BigPK()
    sha256 = SHA256Field(unique=True)
    file_type = pw.ForeignKeyField(FileType)
    file_size = pw.IntegerField(index=True, help_text='File size in bytes')
    sample_size = pw.IntegerField(index=True, null=True, verbose_name='Size of sample in bytes', help_text='This refers to a slightly downscaled copy of the original image (the "sample" image). If it does not exist, this is NULL.')
    thumbnail_size = pw.IntegerField(index=True, null=True, verbose_name='Size of thumbnail in bytes', help_text='This refers to a very downscaled copy of the original image (the "sample" image). If it does not exist, this is NULL.')
    

class Image(PostProperty):
    '''An image is a file attached to a post that is the "content" of the post.'''
    url = pw.CharField(unique=True, verbose_name='Source image URL', help_text='URL where the original image can be downloaded from.')
    downloaded_image = pw.ForeignKeyField(DownloadedImage)

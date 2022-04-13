from typing import Optional
from database import Tag, STALE_THRESHOLD
import net
import datetime

def get_tag_by_json(json_data: dict) -> Tag:
    """
    Create or update a tag from its API JSON representation, and return it.
    """
    # Data as described at https://danbooru.donmai.us/wiki_pages/api%3Atags
    tag = Tag.get_or_none(Tag.id == json_data['id'])
    row_exists = tag is not None
    if tag is None:
        tag = Tag()
        tag.id = json_data['id']
    tag.name = json_data['name']
    tag.category = json_data['category']
    tag.post_count = json_data['post_count']
    tag.is_locked = json_data['is_locked']
    tag.is_deprecated = json_data['is_deprecated']
    tag.created_at = json_data['created_at']
    tag.updated_at = json_data['updated_at']
    
    tag.record_updated_at = datetime.datetime.now()
    tag.save(force_insert=not row_exists)
    return tag

def download_tag_data_by_name(name: str) -> Optional[dict]:
    """
    Download a tag's data from the server, and return it as a dict.
    If the tag doesn't exist on the server, return None.
    """
    response = net.get(f"/tags.json", params={'search[name]': name})
    answers = response.json()
    if len(answers) == 0:
        return None
    return answers[0]

def download_tag_data_by_id(id: int) -> Optional[dict]:
    """
    Download a tag's data from the server, and return it as a dict.
    If the tag doesn't exist on the server, return None.
    """
    response = net.get(f"/tags/{id}.json")
    if response.status_code == 404:
        return None
    return response.json()

def get_tag_by_name(name: str) -> Tag:
    """
    Get a tag by its name.
    If it was last updated too long ago, download it.
    If it doesn't exist in the database, download it.
    If it doesn't exist on the server, raise ValueError.
    """

    tag = Tag.get_or_none(Tag.name == name)
    needs_downloading = False
    if tag is None:
        needs_downloading = True
    elif tag.record_updated_at < datetime.datetime.now() - STALE_THRESHOLD:
        needs_downloading = True

    if needs_downloading:
        if tag:
            tag_data = download_tag_data_by_id(tag.id)
        else:
            tag_data = download_tag_data_by_name(name)
        
        if tag_data is None:
            raise ValueError(f"Tag {name} doesn't exist on the server.")
        
        tag = get_tag_by_json(tag_data)
    
    return tag
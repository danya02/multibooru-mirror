-- This records the latest state of an entity.
-- This should not be part of the media assets database, but it is for now.
CREATE TABLE IF NOT EXISTS latest_state (
    booru_id INTEGER NOT NULL,
    entity_type_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    entity_data_json TEXT NOT NULL,
    PRIMARY KEY (booru_id, entity_type_id, entity_id)
);

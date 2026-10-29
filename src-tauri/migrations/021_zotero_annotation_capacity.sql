-- Expand ResearchClip read cache only; preserve exact original JSON and scoped identities.
-- Read-only Zotero data cached inside ResearchClip, never a Zotero database table.
CREATE TABLE zotero_annotation_cache_v21 (
 document_type TEXT NOT NULL DEFAULT 'document' CHECK(document_type='document'),
 document_id TEXT NOT NULL,
 instance_id TEXT NOT NULL CHECK(length(instance_id)=32),
 library_type TEXT NOT NULL CHECK(library_type IN ('user','group')),
 library_id INTEGER NOT NULL CHECK(library_id>=0),
 attachment_key TEXT NOT NULL CHECK(length(attachment_key)=8),
 content_hash TEXT NOT NULL CHECK(length(content_hash)=64),
 format_version INTEGER NOT NULL CHECK(format_version=1),
 fetched_at INTEGER NOT NULL CHECK(fetched_at>0),
 snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json) AND length(cast(snapshot_json AS BLOB))<=67108864),
 PRIMARY KEY(document_id,instance_id,library_type,library_id,attachment_key,content_hash),
 FOREIGN KEY(document_type,document_id) REFERENCES research_entities(resource_type,id)
) STRICT;

INSERT INTO zotero_annotation_cache_v21 SELECT * FROM zotero_annotation_cache;
DROP TABLE zotero_annotation_cache;
ALTER TABLE zotero_annotation_cache_v21 RENAME TO zotero_annotation_cache;

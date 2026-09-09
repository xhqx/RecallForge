BEGIN IMMEDIATE;
CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS projects(id TEXT PRIMARY KEY,name TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY,project TEXT NOT NULL REFERENCES projects(id));
CREATE TABLE IF NOT EXISTS entries(
 id TEXT PRIMARY KEY,project TEXT NOT NULL REFERENCES projects(id),revision INTEGER NOT NULL,
 status TEXT NOT NULL,kind TEXT NOT NULL,title TEXT NOT NULL,search_text TEXT NOT NULL,
 fingerprint TEXT NOT NULL,json TEXT NOT NULL,created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS entries_project ON entries(project,status);
CREATE INDEX IF NOT EXISTS entries_fingerprint ON entries(project,fingerprint);
CREATE TABLE IF NOT EXISTS revisions(project TEXT NOT NULL,id TEXT NOT NULL REFERENCES entries(id),revision INTEGER NOT NULL,json TEXT NOT NULL,PRIMARY KEY(id,revision));
CREATE TABLE IF NOT EXISTS links(project TEXT NOT NULL REFERENCES projects(id),source TEXT NOT NULL REFERENCES entries(id),target TEXT NOT NULL REFERENCES entries(id),relation TEXT NOT NULL,PRIMARY KEY(source,target,relation));
CREATE TABLE IF NOT EXISTS vector_chunks(rowid INTEGER PRIMARY KEY AUTOINCREMENT,entry_id TEXT NOT NULL REFERENCES entries(id));
CREATE INDEX IF NOT EXISTS chunks_entry ON vector_chunks(entry_id);
CREATE VIRTUAL TABLE IF NOT EXISTS vectors USING vec0(embedding float[384] distance_metric=cosine,project text,status text);
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(title,search_text,content='entries',content_rowid='rowid',tokenize='unicode61');
CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
 INSERT INTO entries_fts(rowid,title,search_text) VALUES(new.rowid,new.title,new.search_text);
END;
CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
 INSERT INTO entries_fts(entries_fts,rowid,title,search_text) VALUES('delete',old.rowid,old.title,old.search_text);
END;
CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
 INSERT INTO entries_fts(entries_fts,rowid,title,search_text) VALUES('delete',old.rowid,old.title,old.search_text);
 INSERT INTO entries_fts(rowid,title,search_text) VALUES(new.rowid,new.title,new.search_text);
END;
PRAGMA user_version=1;
PRAGMA application_id=1380339538;
COMMIT;

use rusqlite::named_params;

use crate::types::{AlreadyExistsError, Collection, DataType, NotFoundError, Project, Tag};
use std::{collections::HashSet, path::PathBuf};

fn get_db() -> rusqlite::Connection {
    let home = dirs::home_dir().unwrap();
    let config_path = home.join(".config/folder_finder");
    let db_path = config_path.join("db.sqlite");

    rusqlite::Connection::open(db_path).unwrap()
}

fn exists(data: &DataType) -> bool {
    let conn = get_db();
    let (table, column, value) = match data {
        DataType::Collection(c) => ("collections", "path", c.path.clone()),
        DataType::Project(p) => ("projects", "path", p.path.clone()),
        DataType::Tag(t) => ("tags", "name", t.name.clone()),
    };
    let stmt = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = '{}')",
        table, column, value
    );
    conn.query_row(stmt.as_str(), [], |row| row.get(0)).unwrap()
}

pub fn init(reset: bool) {
    let home = dirs::home_dir().expect("Could not find home directory");
    // TODO: platform agnostic path
    let config_path = home.join(".config/folder_finder");
    let db_path = config_path.join("db.sqlite");

    if !config_path.exists() {
        println!("Creating config folder");
        std::fs::create_dir_all(&config_path)
            .expect("Could not create config folder in '~/.config'");
    }

    if reset {
        if db_path.exists() {
            println!("Resetting database\nMoving current database to backup");
            let backup_path = config_path.join(format!(
                "db_{}.sqlite.bak",
                chrono::Local::now().format("%Y-%m-%d")
            ));
            std::fs::rename(&db_path, &backup_path)
                .expect("Failed to rename database file. Make sure you have the righ permissions");
        }
    }
    if !db_path.exists() {
        println!("Creating new database");
        let conn = rusqlite::Connection::open(db_path).expect("Failed to create database");
        conn.execute_batch(
            "CREATE TABLE collections (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL
            );
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                collection_id INTEGER,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL
            );
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE project_tags (
                data_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                FOREIGN KEY (data_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            CREATE TABLE collection_tags (
                data_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                FOREIGN KEY (data_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );",
        )
        .expect("Failed to create tables");
    }
}

pub fn get_id(data: &DataType) -> i64 {
    let conn = get_db();
    let (table, column, value) = match data {
        DataType::Collection(c) => ("collections", "path", c.path.clone()),
        DataType::Project(p) => ("projects", "path", p.path.clone()),
        DataType::Tag(t) => ("tags", "name", t.name.clone()),
    };
    let stmt = format!("SELECT id FROM {} WHERE {} = '{}'", table, column, value);
    conn.query_row(stmt.as_str(), [], |row| row.get(0)).unwrap()
}

pub fn add(data: &DataType) -> Result<(), AlreadyExistsError> {
    let conn = get_db();
    if exists(data) {
        return Err(AlreadyExistsError);
    }
    let (table, columns, values) = match data {
        DataType::Collection(c) => ("collections", "path", format!("{}", c.path)),
        DataType::Project(p) => (
            "projects",
            "name, path",
            format!("{}', '{}", p.name, p.path),
        ),
        DataType::Tag(t) => ("tags", "name", format!("{}", t.name)),
    };
    let stmt = format!("INSERT INTO {} ({}) VALUES ('{}')", table, columns, values);
    conn.execute(stmt.as_str(), []).unwrap();
    Ok(())
}

/// Delete and entity(e.g. project or tag) from database along with all its links
pub fn delete(data: &DataType) -> Result<(), rusqlite::Error> {
    let conn = get_db();
    let (table, column, value) = match data {
        DataType::Collection(c) => ("collections", "path", c.path.clone()),
        DataType::Project(p) => ("projects", "path", p.path.clone()),
        DataType::Tag(t) => ("tags", "name", t.name.clone()),
    };
    if !exists(data) {
        return Ok(());
    }
    let res = conn.execute(
        "DELETE FROM :table WHERE :column = :value",
        named_params! {
            ":table": table,
            ":column": column,
            ":value": value,
        },
    );
    // NOTE: linked tags will be deleted by the database due to the CASCADE constraint
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn list_collections() -> Result<Vec<Collection>, rusqlite::Error> {
    let conn = get_db();
    let mut stmt = conn.prepare("SELECT id, path FROM collections").unwrap();
    let mut rows = stmt.query([])?;
    let mut collections = Vec::new();

    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;
        let tags = get_collection_tags(id);
        collections.push(Collection {
            path,
            tags: tags.into_iter().collect(),
        });
    }
    Ok(collections)
}

pub fn list_tags() -> Vec<Tag> {
    let conn = get_db();
    let mut stmt = conn.prepare("SELECT name FROM tags").unwrap();
    let tag_names = stmt.query_map([], |row| row.get(0)).unwrap();
    tag_names.map(|t| Tag { name: t.unwrap() }).collect()
}

pub fn add_tag(data: &DataType, tag: Tag, force: bool) -> Result<(), NotFoundError> {
    let conn = get_db();
    match data {
        DataType::Tag(p) => {
            panic!("Cannot add tag to tag")
        }
        _ => {}
    }
    if !exists(&data) {
        return match data {
            DataType::Collection(_) => Err(NotFoundError::Collection),
            DataType::Project(_) => Err(NotFoundError::Project),
            _ => unreachable!(),
        };
    }
    let tag = DataType::Tag(tag);
    if !exists(&tag) {
        if !force {
            return Err(NotFoundError::Tag);
        }
        add(&tag).unwrap();
    }
    let data_id = get_id(data);
    let tag_id = get_id(&tag);
    let table = match data {
        DataType::Collection(_) => "collection_tags",
        DataType::Project(_) => "project_tags",
        _ => unreachable!(),
    };
    // NOTE: No need to check if link already exists, since the database will ignore duplicates,
    // since it's a many-to-many relationship
    let stmt = format!(
        "INSERT INTO {} (data_id, tag_id) VALUES ({}, {})",
        table, data_id, tag_id
    );
    let _ = conn.execute(stmt.as_str(), []);
    Ok(())
}

fn get_project_tags(project_id: i64) -> HashSet<Tag> {
    let conn = get_db();
    let mut stmt = conn
        .prepare(
            "SELECT tags.name FROM tags
            INNER JOIN project_tags ON tags.id = project_tags.tag_id
            WHERE project_tags.data_id = ?",
        )
        .unwrap();
    let tag_names = stmt.query_map([project_id], |row| row.get(0)).unwrap();
    let mut tags: HashSet<Tag> = tag_names.map(|t| Tag { name: t.unwrap() }).collect();

    let part_of_collection: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ? AND collection_id IS NOT NULL)",
            [project_id],
            |row| row.get(0),
        )
        .unwrap();
    if !part_of_collection {
        return tags;
    }

    let collection_id: i64 = conn
        .query_row(
            "SELECT collection_id FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .unwrap();

    tags.union(&get_collection_tags(collection_id))
        .cloned()
        .collect()
}

fn get_collection_tags(collection_id: i64) -> HashSet<Tag> {
    let conn = get_db();
    let mut stmt = conn
        .prepare(
            format!(
                "SELECT tags.name FROM tags
            INNER JOIN collection_tags ON tags.id = collection_tags.tag_id
            WHERE collection_tags.data_id = {}",
                collection_id
            )
            .as_str(),
        )
        .unwrap();
    let tag_names = stmt.query_map([], |row| row.get(0)).unwrap();
    tag_names.map(|t| Tag { name: t.unwrap() }).collect()
}

pub fn add_project(path: PathBuf) -> Result<(), AlreadyExistsError> {
    let conn = get_db();
    let exists: bool = conn
        .query_row(
            "EXISTS(SELECT 1 FROM projects WHERE path = ?)",
            [path.to_str().unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    if exists {
        return Err(AlreadyExistsError);
    }
    conn.execute(
        "INSERT INTO projects (name, path) VALUES (?, ?)",
        [
            path.file_name().unwrap().to_str().unwrap(),
            path.to_str().unwrap(),
        ],
    )
    .unwrap();
    Ok(())
}

pub fn list_projects() -> Result<Vec<Project>, rusqlite::Error> {
    let conn = get_db();
    let mut stmt = conn.prepare("SELECT id, name, path FROM projects").unwrap();
    let mut rows = stmt.query([])?;
    let mut projects = Vec::new();

    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let path: String = row.get(2)?;
        let tags = get_project_tags(id);
        projects.push(Project {
            name,
            path,
            collection: None,
            tags: tags.into_iter().collect(),
        });
    }
    Ok(projects)
}

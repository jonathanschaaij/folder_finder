use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub path: String,
    pub tags: HashSet<Tag>,
}

impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tagsstr = self
            .tags
            .iter()
            .fold(String::new(), |acc, tag| acc + &tag.name + " ");
        write!(f, "{} | {}", self.path, tagsstr.trim_end())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub collection: Option<Collection>,
    pub tags: HashSet<Tag>,
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tagsstr = self
            .tags
            .iter()
            .fold(String::new(), |acc, tag| acc + &tag.name + " ");
        write!(f, "{} | {} | {}", self.name, self.path, tagsstr.trim_end())
    }
}

pub enum DataType {
    Collection(Collection),
    Project(Project),
    Tag(Tag),
}
pub enum EmptyDataType {
    Collection,
    Project,
    Tag,
}

#[derive(Debug, Clone)]
pub struct AlreadyExistsError;

#[derive(Debug, Clone)]
pub struct DatabaseError;

#[derive(Debug, Clone)]
pub enum NotFoundError {
    Collection,
    Project,
    Tag,
}

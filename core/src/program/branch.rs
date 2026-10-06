use super::tree::Tree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch<'a> {
    pub tree: &'a Tree,
    pub trail: Vec<&'a Tree>,
    pub prefix: String,
}

impl Tree {
    pub fn branch(&self, uuid: &str) -> Option<Branch<'_>> {
        if self.program.uuid == uuid {
            return Some(Branch {
                tree: self,
                trail: Vec::new(),
                prefix: String::new(),
            });
        }
        self.children.iter().find_map(|(directory, child)| {
            let mut branch = child.branch(uuid)?;
            branch.trail.insert(0, self);
            branch.prefix = format!("children/{directory}/{}", branch.prefix);
            Some(branch)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactNode {
    name: String,
    children: Vec<Box<ArtifactNode>>,
}

impl ArtifactNode {
    pub fn new(name: &str) -> ArtifactNode {
        ArtifactNode {
            name: name.to_string(),
            children: Vec::<Box<ArtifactNode>>::new(),
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&mut ArtifactNode> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<ArtifactNode>,
    {
        self.children.push(Box::new(leaf.into()));
        self
    }
}

pub fn build_artifact_tree(node: &mut ArtifactNode, parts: &Vec<&str>, depth: usize) {
    if depth >= parts.len() {
        // Finished building the tree
        return;
    }
    let part = &parts[depth];
    let mut child_node = match node.find_child(&part) {
        Some(dir) => dir,
        None => {
            // Create a new child node and return it
            let new_node = ArtifactNode::new(&part);
            node.add_child(new_node);
            node.find_child(&part).unwrap()
        }
    };
    build_artifact_tree(&mut child_node, parts, depth + 1);
}

fn print_file(file_name: &str, depth: u32) {
    if depth == 0 {
        println!("{}", file_name);
    } else {
        println!(
            "{:indent$}{} {}",
            "",
            "└──",
            file_name,
            indent = (depth * 4) as usize
        );
    }
}

pub fn print_artifact_tree(node: &ArtifactNode, depth: u32) {
    print_file(&node.name, depth);
    for child in &node.children {
        print_artifact_tree(&child, depth + 1);
    }
}

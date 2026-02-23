use std::collections::HashMap;

#[derive(Debug)]
pub struct BkNode {
    pub term: String,
    pub children: HashMap<usize, Box<BkNode>>,
    pub cand_ids: Vec<usize>,
}

#[derive(Debug)]
pub struct BkTree {
    pub root: Option<Box<BkNode>>,
}

impl BkTree {
    pub fn new() -> Self { Self { root: None } }

    pub fn insert<F>(&mut self, term: String, cand_id: usize, dist: &F)
    where
        F: Fn(&str, &str) -> usize,
    {
        match self.root.as_mut() {
            None => {
                self.root = Some(Box::new(BkNode {
                    term,
                    children: HashMap::new(),
                    cand_ids: vec![cand_id],
                }));
            }
            Some(root) => Self::insert_into(root, term, cand_id, dist),
        }
    }

    fn insert_into<F>(node: &mut BkNode, term: String, cand_id: usize, dist: &F)
    where
        F: Fn(&str, &str) -> usize,
    {
        let d = dist(node.term.as_str(), term.as_str());
        if d == 0 {
            node.cand_ids.push(cand_id);
            return;
        }
        match node.children.get_mut(&d) {
            Some(child) => Self::insert_into(child, term, cand_id, dist),
            None => {
                node.children.insert(d, Box::new(BkNode {
                    term,
                    children: HashMap::new(),
                    cand_ids: vec![cand_id],
                }));
            }
        }
    }

    pub fn search<F>(&self, query: &str, radius: usize, out: &mut Vec<usize>, dist: &F)
    where
        F: Fn(&str, &str) -> usize,
    {
        if let Some(root) = self.root.as_ref() {
            Self::search_node(root, query, radius, out, dist);
        }
    }

    fn search_node<F>(node: &BkNode, query: &str, radius: usize, out: &mut Vec<usize>, dist: &F)
    where
        F: Fn(&str, &str) -> usize,
    {
        let d = dist(node.term.as_str(), query);
        if d <= radius {
            out.extend_from_slice(&node.cand_ids);
        }
        let lo = d.saturating_sub(radius);
        let hi = d + radius;
        for (edge, child) in node.children.iter() {
            if *edge >= lo && *edge <= hi {
                Self::search_node(child, query, radius, out, dist);
            }
        }
    }
}
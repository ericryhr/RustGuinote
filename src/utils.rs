use std::collections::HashSet;

pub fn intersect<T: Eq + std::hash::Hash + Clone>(vec1: &Vec<T>, vec2: &Vec<T>) -> Vec<T> {
    let set1: HashSet<_> = vec1.iter().cloned().collect();
    let set2: HashSet<_> = vec2.iter().cloned().collect();
    
    set1.intersection(&set2).cloned().collect()
}
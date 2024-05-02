/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut res: Vec<i32> = Vec::new();
    for x in v {
        res.push(x + n);
    }
    res
}

fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for x in v {
        *x += n;
    }
}

fn dedup(v: &mut Vec<i32>) {
    let mut seen = HashSet::new();
    let mut i = 0;
    let mut j = 0;

    while j < v.len() {
        if seen.insert(v[j]) {
            // If the element is not a duplicate, keep it at position i
            v[i] = v[j];
            i += 1;
        }
        j += 1;
    }

    // Truncate the vector to remove any extra elements after deduplication
    v.truncate(i);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}

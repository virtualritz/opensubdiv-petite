//! Tests for the Index newtype wrapper.

use opensubdiv_petite::Index;

#[test]
fn test_index_from_u32() {
    let idx = Index::from(42u32);
    assert_eq!(idx.0, 42);
}

#[test]
fn test_index_into_u32() {
    let idx = Index(42);
    let value: u32 = idx.into();
    assert_eq!(value, 42);
}

#[test]
fn test_index_from_usize() {
    let idx = Index::from(100usize);
    assert_eq!(idx.0, 100);
}

#[test]
fn test_index_into_usize() {
    let idx = Index(100);
    let value: usize = idx.into();
    assert_eq!(value, 100);
}

#[test]
fn test_index_clone() {
    let idx1 = Index(42);
    let idx2 = idx1.clone();
    assert_eq!(idx1, idx2);
}

#[test]
fn test_index_copy() {
    let idx1 = Index(42);
    let idx2 = idx1; // Copy
    assert_eq!(idx1, idx2);
}

#[test]
fn test_index_debug() {
    let idx = Index(42);
    let debug_str = format!("{:?}", idx);
    assert_eq!(debug_str, "Index(42)");
}

#[test]
fn test_index_equality() {
    let idx1 = Index(42);
    let idx2 = Index(42);
    let idx3 = Index(43);

    assert_eq!(idx1, idx2);
    assert_ne!(idx1, idx3);
}

#[test]
fn test_index_ordering() {
    let idx1 = Index(1);
    let idx2 = Index(2);
    let idx3 = Index(2);

    assert!(idx1 < idx2);
    assert!(idx2 > idx1);
    assert!(idx2 <= idx3);
    assert!(idx2 >= idx3);
}

#[test]
fn test_index_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(Index(1), "one");
    map.insert(Index(2), "two");

    assert_eq!(map.get(&Index(1)), Some(&"one"));
    assert_eq!(map.get(&Index(2)), Some(&"two"));
    assert_eq!(map.get(&Index(3)), None);
}

#[test]
fn test_index_in_vec() {
    let indices = vec![Index(0), Index(1), Index(2)];

    assert_eq!(indices[0].0, 0);
    assert_eq!(indices[1].0, 1);
    assert_eq!(indices[2].0, 2);
}

#[test]
fn test_index_arithmetic() {
    let idx1 = Index(10);
    let idx2 = Index(5);

    // Can access the inner value for arithmetic.
    let sum = idx1.0 + idx2.0;
    assert_eq!(sum, 15);

    let diff = idx1.0 - idx2.0;
    assert_eq!(diff, 5);
}

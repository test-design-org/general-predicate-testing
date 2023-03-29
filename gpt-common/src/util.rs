pub trait UniquesVec<T> {
    fn uniques(&self) -> Vec<T>;
}

impl<T: PartialEq + Clone> UniquesVec<T> for Vec<T> {
    /// Returns a Vec with only unique elements compared by `PartialEq`
    ///
    /// O(n^2) complexity, because it can't use `Ord`, so it has to compare each element to each other.
    /// Also clones every element.
    // TODO: This could be done in place
    fn uniques(&self) -> Self {
        let mut uniques = Self::new();

        for x in self.iter() {
            let contains = uniques.iter().any(|y| x == y);
            if !contains {
                uniques.push(x.clone());
            }
        }

        uniques
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};
    use rstest::rstest;

    use super::UniquesVec;

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::unique(vec![1, 2, 3, 4], vec![1, 2, 3, 4])]
    #[case::non_uniques(
        vec![1, 2, 3, 1, 1, 1, 2, 1, 3, 3, 2, 1, 3, 2, 4],
        vec![1, 2, 3, 4],
    )]
    fn test_unique(#[case] input: Vec<u8>, #[case] expected: Vec<u8>) {
        assert_eq!(input.uniques(), expected);
    }
}

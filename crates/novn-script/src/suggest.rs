pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut before: Vec<usize> = Vec::new();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];

    for i in 1..=a.len() {
        current[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(before[j - 2] + 1);
            }
            current[j] = best;
        }
        before = std::mem::replace(&mut previous, current.clone());
    }

    previous[b.len()]
}

pub fn closest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (word.chars().count() / 3).max(1);

    candidates
        .into_iter()
        .filter(|&candidate| candidate != word)
        .map(|candidate| (edit_distance(word, candidate), candidate))
        .filter(|&(distance, _)| distance <= limit)
        .min()
        .map(|(_, candidate)| candidate)
}

pub fn did_you_mean<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> String {
    match closest(word, candidates) {
        Some(candidate) => format!("; did you mean '{}'?", candidate),
        None => String::new(),
    }
}

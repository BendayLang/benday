/// This fuzzy finder will return all the elements in the order of their similarity to the query.
pub fn hard_fuzzy_find(elements: Vec<String>, query: &str) -> Vec<String> {
	// TODO make it case insensitive
	let corpus = ngrammatic::CorpusBuilder::new().fill(elements).finish();
	let mut res: Vec<(usize, String)> =
		corpus.search(query, 0.).iter().map(|r| ((r.similarity * 1000.) as usize, r.text.clone())).collect();
	res.sort();
	return res.iter().rev().map(|r| r.1.clone()).collect();
}

/// This fuzzy finder will return all the elements that match the simple pattern of the query. // TODO clarify
pub fn easy_fuzzy_find(elements: Vec<String>, query: &str) -> Vec<String> {
	if elements.is_empty() {
		return Vec::new();
	}
	let mut res: Vec<(usize, String)> = Vec::new();
	for el in elements {
		let count = does_match(&el.to_ascii_lowercase(), &query.to_ascii_lowercase());
		if let Some(count) = count {
			res.push((count, el));
		}
	}
	res.sort();
	return res.iter().map(|r| r.1.clone()).collect();
}

fn does_match(element: &str, query: &str) -> Option<usize> {
	let mut count = 0;
	let mut element = element;
	for c in query.chars() {
		let found = element.find(c);
		if let Some(index) = found {
			count += index;
			element = &element[(index + 1)..];
		} else {
			return None;
		}
	}
	Some(count)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic() {
		let res = easy_fuzzy_find(
			vec!["var 2".to_string(), "var 3".to_string(), "v01".to_string(), "vAr 1".to_string(), "v1".to_string()],
			"v1",
		);
		assert_eq!(res, vec!["v1".to_string(), "v01".to_string(), "vAr 1".to_string(),]);
	}

	#[test]
	fn hard_f() {
		let res = hard_fuzzy_find(
			vec![
				"var 2".to_string(),
				"var 3".to_string(),
				"violoncelle".to_string(),
				"VAR".to_string(),
				"v01".to_string(),
				"vAr 1".to_string(),
				"v1".to_string(),
			],
			"v1",
		);
		assert_eq!(
			res,
			vec![
				"v1".to_string(),
				"v01".to_string(),
				"vAr 1".to_string(),
				"var 3".to_string(),
				"var 2".to_string(),
				"violoncelle".to_string(),
			]
		);
	}
}

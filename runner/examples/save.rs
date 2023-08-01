use runner::exectute::{load_ast_from, save_ast_to};

fn save() {
	let ast = load_ast_from(&std::path::Path::new("models/examples/ast.json")).expect("Failed to load ast.json");
	save_ast_to(&ast, &std::path::Path::new("runner/examples/ast.json")).unwrap();
	let ast2 = load_ast_from(&std::path::Path::new("runner/examples/ast.json")).unwrap();
	assert_eq!(ast, ast2);
}

fn main() {
	save();
}

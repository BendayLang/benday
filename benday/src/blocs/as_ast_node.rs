use models::ast;
use pg_sdl::widgets::Manager;

pub trait AsAstNode {
	fn as_ast_node(&self, manager: &Manager) -> ast::Node;
}

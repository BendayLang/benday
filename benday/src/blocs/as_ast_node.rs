use models::ast;
use pg_sdl::widgets::{Manager, WidgetId};

pub trait AsAstNode {
	fn as_ast_node(&self, manager: &Manager) -> ast::Node;
}

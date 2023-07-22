use models::ast;
use pg_sdl::widgets::{WidgetId, WidgetsManager};

pub trait AsAstNode {
	fn as_ast_node(&self, widgets_manager: &WidgetsManager) -> ast::Node;
}

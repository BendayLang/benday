use models::ast;
use pg_sdl::widgets::{WidgetId, WidgetsManager};

pub trait AsAstNode {
	fn as_ast_node(&self, blocs: &Vec<WidgetId>, widgets_manager: &WidgetsManager) -> ast::Node;
}

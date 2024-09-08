use crate::expr::Expr;

pub fn ast_print(expr: Expr) -> String {
    match expr {
        Expr::Binary(expr) => {
            Expr::parenthesize(expr.operator.lexeme, vec![*expr.left, *expr.right])
        }
        Expr::Grouping(expr) => Expr::parenthesize("group".to_string(), vec![*expr.expr]),
        Expr::Literal(expr) => Expr::parenthesize(expr.literal.to_string(), Vec::new()),
        Expr::Unary(expr) => Expr::parenthesize(expr.operator.lexeme, vec![*expr.right]),
    }
}
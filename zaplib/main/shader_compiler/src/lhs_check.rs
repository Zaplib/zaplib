use {
    crate::{
        env::{Env, Sym, VarKind},
        error::ParseError,
        ident::{Ident, IdentPath},
        lit::Lit,
        shaderast::*,
        span::Span,
        ty::TyLit,
    },
    std::cell::Cell,
};

pub(crate) struct LhsChecker<'a> {
    pub(crate) env: &'a Env,
}

impl<'a> LhsChecker<'a> {
    pub(crate) fn lhs_check_expr(&mut self, expr: &Expr) -> Result<(), ParseError> {
        match expr.kind {
            ExprKind::Cond { span, ref expr, ref expr_if_true, ref expr_if_false, .. } => {
                self.lhs_check_cond_expr(span, expr, expr_if_true, expr_if_false)
            }
            ExprKind::Bin { span, op, ref left_expr, ref right_expr, .. } => {
                self.lhs_check_bin_expr(span, op, left_expr, right_expr)
            }
            ExprKind::Un { span, op, ref expr } => self.lhs_check_un_expr(span, op, expr),
            ExprKind::MethodCall { span, ident, ref arg_exprs } => self.lhs_check_method_call_expr(span, ident, arg_exprs),
            ExprKind::Field { span, ref expr, field_ident } => self.lhs_check_field_expr(span, expr, field_ident),
            ExprKind::Index { span, ref expr, ref index_expr } => self.lhs_check_index_expr(span, expr, index_expr),
            ExprKind::Call { span, ident_path, ref arg_exprs } => self.lhs_check_call_expr(span, ident_path, arg_exprs),
            ExprKind::ConsCall { span, ty_lit, ref arg_exprs } => self.lhs_check_cons_call_expr(span, ty_lit, arg_exprs),
            ExprKind::Var { span, ref kind, ident_path } => self.lhs_check_var_expr(span, kind, ident_path),
            ExprKind::Lit { span, lit } => self.lhs_check_lit_expr(span, lit),
        }
    }

    fn lhs_check_cond_expr(
        &mut self,
        span: Span,
        _expr: &Expr,
        _expr_if_true: &Expr,
        _expr_if_false: &Expr,
    ) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_bin_expr(&mut self, span: Span, _op: BinOp, _left_expr: &Expr, _right_expr: &Expr) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_un_expr(&mut self, span: Span, _op: UnOp, _expr: &Expr) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_method_call_expr(&mut self, span: Span, _ident: Ident, _arg_exprs: &[Expr]) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_field_expr(&mut self, _span: Span, expr: &Expr, _field_ident: Ident) -> Result<(), ParseError> {
        self.lhs_check_expr(expr)
    }

    fn lhs_check_index_expr(&mut self, _span: Span, expr: &Expr, _index_expr: &Expr) -> Result<(), ParseError> {
        self.lhs_check_expr(expr)
    }

    fn lhs_check_call_expr(&mut self, span: Span, _ident_path: IdentPath, _arg_exprs: &[Expr]) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_cons_call_expr(&mut self, span: Span, _ty_lit: TyLit, _arg_exprs: &[Expr]) -> Result<(), ParseError> {
        Err(ParseError { span, message: String::from("expression is not a valid left hand side") })
    }

    fn lhs_check_var_expr(&mut self, span: Span, _kind: &Cell<Option<VarKind>>, ident_path: IdentPath) -> Result<(), ParseError> {
        match self.env.find_sym(ident_path).unwrap() {
            Sym::Var { is_mut, .. } => {
                if !is_mut {
                    return Err(ParseError { span, message: String::from("expression is not a valid left hand side") });
                }
                Ok(())
            }
            _ => panic!(),
        }
    }

    fn lhs_check_lit_expr(&mut self, _span: Span, _lit: Lit) -> Result<(), ParseError> {
        Ok(())
    }
}

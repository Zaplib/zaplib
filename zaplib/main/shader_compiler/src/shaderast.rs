use crate::env::VarKind;
use crate::ident::{Ident, IdentPath};
use crate::lit::Lit;
use crate::span::Span;
use crate::ty::{Ty, TyExpr, TyLit};
use crate::val::Val;
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct ShaderAst {
    pub debug: bool,
    pub decls: Vec<Decl>,
}

impl ShaderAst {
    pub(crate) fn find_geometry_decl(&self, ident: Ident) -> Option<&GeometryDecl> {
        self.decls.iter().find_map(|decl| {
            match decl {
                Decl::Geometry(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident == ident)
        })
    }

    pub(crate) fn find_const_decl(&self, ident: Ident) -> Option<&ConstDecl> {
        self.decls.iter().find_map(|decl| {
            match decl {
                Decl::Const(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident == ident)
        })
    }

    pub(crate) fn find_fn_decl(&self, ident_path: IdentPath) -> Option<&FnDecl> {
        self.decls.iter().rev().find_map(|decl| {
            match decl {
                Decl::Fn(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident_path == ident_path)
        })
    }

    pub(crate) fn find_instance_decl(&self, ident: Ident) -> Option<&InstanceDecl> {
        self.decls.iter().find_map(|decl| {
            match decl {
                Decl::Instance(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident == ident)
        })
    }

    pub(crate) fn find_struct_decl(&self, ident: Ident) -> Option<&StructDecl> {
        self.decls.iter().find_map(|decl| {
            match decl {
                Decl::Struct(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident == ident)
        })
    }

    pub(crate) fn find_uniform_decl(&self, ident: Ident) -> Option<&UniformDecl> {
        self.decls.iter().find_map(|decl| {
            match decl {
                Decl::Uniform(decl) => Some(decl),
                _ => None,
            }
            .filter(|decl| decl.ident == ident)
        })
    }
}

#[derive(Clone, Debug)]
pub enum Decl {
    Geometry(GeometryDecl),
    Const(ConstDecl),
    Fn(FnDecl),
    Instance(InstanceDecl),
    Struct(StructDecl),
    Texture(TextureDecl),
    Uniform(UniformDecl),
    Varying(VaryingDecl),
}

#[derive(Clone, Debug)]
pub struct GeometryDecl {
    pub(crate) is_used_in_fragment_shader: Cell<Option<bool>>,
    pub(crate) span: Span,
    pub ident: Ident,
    pub ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub struct ConstDecl {
    pub(crate) span: Span,
    pub(crate) ident: Ident,
    pub(crate) ty_expr: TyExpr,
    pub(crate) expr: Expr,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub(crate) span: Span,
    pub(crate) return_ty: RefCell<Option<Ty>>,
    pub(crate) is_used_in_vertex_shader: Cell<Option<bool>>,
    pub(crate) is_used_in_fragment_shader: Cell<Option<bool>>,
    pub(crate) callees: RefCell<Option<BTreeSet<IdentPath>>>,
    pub(crate) uniform_block_deps: RefCell<Option<BTreeSet<Ident>>>,
    pub(crate) has_texture_deps: Cell<Option<bool>>,
    pub(crate) geometry_deps: RefCell<Option<BTreeSet<Ident>>>,
    pub(crate) instance_deps: RefCell<Option<BTreeSet<Ident>>>,
    pub(crate) has_varying_deps: Cell<Option<bool>>,
    pub(crate) cons_fn_deps: RefCell<Option<BTreeSet<(TyLit, Vec<Ty>)>>>,
    pub(crate) ident_path: IdentPath,
    pub(crate) params: Vec<Param>,
    pub(crate) return_ty_expr: Option<TyExpr>,
    pub(crate) block: Block,
}

#[derive(Clone, Debug)]
pub struct InstanceDecl {
    pub(crate) is_used_in_fragment_shader: Cell<Option<bool>>,
    pub(crate) span: Span,
    pub ident: Ident,
    pub ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub(crate) span: Span,
    pub(crate) ident: Ident,
    pub(crate) fields: Vec<Field>,
}

impl StructDecl {
    pub(crate) fn find_field(&self, ident: Ident) -> Option<&Field> {
        self.fields.iter().find(|field| field.ident == ident)
    }
}

#[derive(Clone, Debug)]
pub struct TextureDecl {
    pub(crate) span: Span,
    pub ident: Ident,
    pub ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub struct UniformDecl {
    pub(crate) span: Span,
    pub ident: Ident,
    pub ty_expr: TyExpr,
    pub block_ident: Option<Ident>,
}

#[derive(Clone, Debug)]
pub struct VaryingDecl {
    pub(crate) span: Span,
    pub(crate) ident: Ident,
    pub(crate) ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub(crate) span: Span,
    pub(crate) is_inout: bool,
    pub(crate) ident: Ident,
    pub(crate) ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub(crate) ident: Ident,
    pub(crate) ty_expr: TyExpr,
}

#[derive(Clone, Debug)]
pub(crate) struct Block {
    pub(crate) stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub(crate) enum Stmt {
    Break { span: Span },
    Continue { span: Span },
    For { span: Span, ident: Ident, from_expr: Expr, to_expr: Expr, step_expr: Option<Expr>, block: Box<Block> },
    If { span: Span, expr: Expr, block_if_true: Box<Block>, block_if_false: Option<Box<Block>> },
    Let { span: Span, ty: RefCell<Option<Ty>>, ident: Ident, ty_expr: Option<TyExpr>, expr: Option<Expr> },
    Return { span: Span, expr: Option<Expr> },
    Expr { span: Span, expr: Expr },
}

#[derive(Clone, Debug)]
pub(crate) struct Expr {
    pub(crate) span: Span,
    pub(crate) ty: RefCell<Option<Ty>>,
    pub(crate) const_val: RefCell<Option<Option<Val>>>,
    pub(crate) const_index: Cell<Option<usize>>,
    pub(crate) kind: ExprKind,
}

#[derive(Clone, Debug)]
pub(crate) enum ExprKind {
    Cond { span: Span, expr: Box<Expr>, expr_if_true: Box<Expr>, expr_if_false: Box<Expr> },
    Bin { span: Span, op: BinOp, left_expr: Box<Expr>, right_expr: Box<Expr> },
    Un { span: Span, op: UnOp, expr: Box<Expr> },
    MethodCall { span: Span, ident: Ident, arg_exprs: Vec<Expr> },
    Field { span: Span, expr: Box<Expr>, field_ident: Ident },
    Index { span: Span, expr: Box<Expr>, index_expr: Box<Expr> },
    Call { span: Span, ident_path: IdentPath, arg_exprs: Vec<Expr> },
    ConsCall { span: Span, ty_lit: TyLit, arg_exprs: Vec<Expr> },
    Var { span: Span, kind: Cell<Option<VarKind>>, ident_path: IdentPath },
    Lit { span: Span, lit: Lit },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BinOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOp::Assign => "=",
                BinOp::AddAssign => "+=",
                BinOp::SubAssign => "-=",
                BinOp::MulAssign => "*=",
                BinOp::DivAssign => "/=",
                BinOp::Or => "||",
                BinOp::And => "&&",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
            }
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum UnOp {
    Not,
    Neg,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnOp::Not => "!",
                UnOp::Neg => "-",
            }
        )
    }
}

use crate::error::ParseError;
use crate::ident::IdentPath;

use crate::span::Span;
use crate::ty::Ty;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct Env {
    scopes: Vec<Scope>,
}

impl Env {
    pub(crate) fn find_sym(&self, ident_path: IdentPath) -> Option<Sym> {
        let ret = self.scopes.iter().rev().find_map(|scope| scope.get(&ident_path));
        if ret.is_some() {
            return Some(ret.unwrap().clone());
        }
        None
    }

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope::new())
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    pub(crate) fn insert_sym(&mut self, span: Span, ident_path: IdentPath, sym: Sym) -> Result<(), ParseError> {
        match self.scopes.last_mut().unwrap().entry(ident_path) {
            Entry::Vacant(entry) => {
                entry.insert(sym);
                Ok(())
            }
            Entry::Occupied(_) => Err(ParseError { span, message: format!("`{}` is already defined in this scope", ident_path) }),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Sym {
    Builtin,
    Fn,
    TyVar { ty: Ty },
    Var { is_mut: bool, ty: Ty, kind: VarKind },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum VarKind {
    Geometry,
    Const,
    Instance,
    Local,
    Texture,
    Uniform,
    Varying,
}

type Scope = HashMap<IdentPath, Sym>;

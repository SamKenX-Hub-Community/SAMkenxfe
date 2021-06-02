use crate::errors::SemanticError;
use crate::namespace::scopes::{BlockScope, Scope, Shared};
use crate::namespace::types::FixedSize;
use crate::traversal::{expressions, types};
use crate::Context;
use fe_parser::ast as fe;
use fe_parser::node::Node;
use std::rc::Rc;

/// Gather context information for var declarations and check for type errors.
pub fn var_decl(
    scope: Shared<BlockScope>,
    context: &mut Context,
    stmt: &Node<fe::FuncStmt>,
) -> Result<(), SemanticError> {
    if let fe::FuncStmt::VarDecl { target, typ, value } = &stmt.kind {
        let declared_type =
            types::type_desc_fixed_size(&Scope::Block(Rc::clone(&scope)), context, &typ)?;

        if let Some(value) = value {
            let value_attributes = expressions::assignable_expr(
                Rc::clone(&scope),
                context,
                value,
                Some(&declared_type.clone().into()),
            )?;

            if declared_type != value_attributes.typ {
                context.type_error(
                    "type mismatch",
                    value.span,
                    &declared_type,
                    value_attributes.typ,
                );
            }
        }

        add_var(&scope, &target, declared_type.clone())?;
        context.add_declaration(stmt, declared_type);
        return Ok(());
    }

    unreachable!()
}

/// Add declared variables to the scope.
fn add_var(
    scope: &Shared<BlockScope>,
    target: &Node<fe::VarDeclTarget>,
    typ: FixedSize,
) -> Result<(), SemanticError> {
    match (&target.kind, typ) {
        (fe::VarDeclTarget::Name(name), typ) => scope.borrow_mut().add_var(name, typ),
        (fe::VarDeclTarget::Tuple(items), FixedSize::Tuple(items_ty)) => {
            let items_ty = items_ty.items;
            if items.len() != items_ty.as_vec().len() {
                return Err(SemanticError::type_error());
            }
            for (item, item_ty) in items.iter().zip(items_ty.into_iter()) {
                add_var(scope, item, item_ty)?;
            }
            Ok(())
        }
        _ => Err(SemanticError::type_error()),
    }
}

// SPDX-License-Identifier: Apache-2.0

use super::ast::*;
use super::diagnostics::Diagnostics;
use super::eval::check_term_for_constant_overflow;
use super::expression::{
    available_functions, call_expr, constructor_named_args, expression, function_call_expr,
    function_call_pos_args, match_constructor_to_args, named_call_expr, named_function_call_expr,
    new, ExprContext, ResolveTo,
};
use super::symtable::{LoopScopes, Symtable};
use crate::sema::builtin;
use crate::sema::symtable::{VariableInitializer, VariableUsage};
use crate::sema::unused_variable::{assigned_variable, check_function_call, used_variable};
use crate::sema::Recurse;
use ola_parser::program;
use ola_parser::program::CodeLocation;
use ola_parser::program::OptionalCodeLocation;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

pub fn resolve_function_body(
    def: &program::FunctionDefinition,
    file_no: usize,
    contract_no: Option<usize>,
    function_no: usize,
    ns: &mut Namespace,
) -> Result<(), ()> {
    let mut symtable = Symtable::new();
    let mut loops = LoopScopes::new();
    let mut res = Vec::new();
    let context = ExprContext {
        file_no,
        contract_no,
        function_no: Some(function_no),
        unchecked: false,
        constant: false,
        lvalue: false,
        yul_function: false,
    };

    // first add function parameters
    for (i, p) in def.params.iter().enumerate() {
        let p = p.1.as_ref().unwrap();
        if let Some(ref name) = p.name {
            if let Some(pos) = symtable.add(
                name,
                ns.functions[function_no].params[i].ty.clone(),
                ns,
                VariableInitializer::Solidity(None),
                VariableUsage::Parameter,
                p.storage.clone(),
            ) {
                ns.check_shadowing(file_no, contract_no, name);

                symtable.arguments.push(Some(pos));
            }
        } else {
            symtable.arguments.push(None);
        }
    }

    // now that the function arguments have been resolved, we can resolve the bases for
    // constructors.
    if def.ty == program::FunctionTy::Constructor {
        let contract_no = contract_no.unwrap();
        let mut resolve_bases: BTreeMap<usize, program::Loc> = BTreeMap::new();
        let mut all_ok = true;
        let mut diagnostics = Diagnostics::default();

        for attr in &def.attributes {
            if let program::FunctionAttribute::BaseOrModifier(loc, base) = attr {
                match ns.resolve_contract_with_namespace(file_no, &base.name, &mut diagnostics) {
                    Ok(base_no) => {
                        if base_no == contract_no || !is_base(base_no, contract_no, ns) {
                            ns.diagnostics.push(Diagnostic::error(
                                *loc,
                                format!(
                                    "contract '{}' is not a base contract of '{}'",
                                    base.name, ns.contracts[contract_no].name,
                                ),
                            ));
                            all_ok = false;
                        } else if let Some(prev) = resolve_bases.get(&base_no) {
                            ns.diagnostics.push(Diagnostic::error_with_note(
                                *loc,
                                format!("duplicate base contract '{}'", base.name),
                                *prev,
                                format!("previous base contract '{}'", base.name),
                            ));
                            all_ok = false;
                        } else if let Some(args) = &base.args {
                            let mut diagnostics = Diagnostics::default();

                            // find constructor which matches this
                            if let Ok((Some(constructor_no), args)) = match_constructor_to_args(
                                &base.loc,
                                args,
                                base_no,
                                &context,
                                ns,
                                &mut symtable,
                                &mut diagnostics,
                            ) {
                                for arg in &args {
                                    used_variable(ns, arg, &mut symtable);
                                }
                                ns.functions[function_no]
                                    .bases
                                    .insert(base_no, (base.loc, constructor_no, args));

                                resolve_bases.insert(base_no, base.loc);
                            }

                            ns.diagnostics.extend(diagnostics);
                        } else {
                            ns.diagnostics.push(Diagnostic::error(
                                *loc,
                                format!(
                                    "missing arguments to constructor of contract '{}'",
                                    base.name
                                ),
                            ));
                            all_ok = false;
                        }
                    }
                    Err(_) => {
                        all_ok = false;
                    }
                }
            }
        }

        if all_ok && ns.contracts[contract_no].instantiable {
            for base in &ns.contracts[contract_no].bases {
                // do we have constructor arguments
                if base.constructor.is_some() || resolve_bases.contains_key(&base.contract_no) {
                    continue;
                }

                // does the contract require arguments
                if ns.contracts[base.contract_no].constructor_needs_arguments(ns) {
                    ns.diagnostics.push(Diagnostic::error(
                        def.loc,
                        format!(
                            "missing arguments to contract '{}' constructor",
                            ns.contracts[base.contract_no].name
                        ),
                    ));
                }
            }
        }

        ns.diagnostics.extend(diagnostics);
    }

    // resolve modifiers on functions
    if def.ty == program::FunctionTy::Function {
        let mut modifiers = Vec::new();
        let mut diagnostics = Diagnostics::default();

        for attr in &def.attributes {
            if let program::FunctionAttribute::BaseOrModifier(_, modifier) = attr {
                if modifier.name.identifiers.len() != 1 {
                    ns.diagnostics.push(Diagnostic::error(
                        def.loc,
                        format!("unknown modifier '{}' on function", modifier.name),
                    ));
                } else {
                    let modifier_name = &modifier.name.identifiers[0];
                    if let Ok(e) = function_call_pos_args(
                        &modifier.loc,
                        modifier_name,
                        program::FunctionTy::Modifier,
                        modifier.args.as_ref().unwrap_or(&Vec::new()),
                        available_functions(
                            &modifier_name.name,
                            false,
                            context.file_no,
                            context.contract_no,
                            ns,
                        ),
                        true,
                        &context,
                        ns,
                        ResolveTo::Unknown,
                        &mut symtable,
                        &mut diagnostics,
                    ) {
                        modifiers.push(e);
                    }
                }
            }
        }

        ns.diagnostics.extend(diagnostics);
        ns.functions[function_no].modifiers = modifiers;
    }

    // a function with no return values does not need a return statement
    let mut return_required = !def.returns.is_empty();

    // If any of the return values are named, then the return statement can be omitted at
    // the end of the function, and return values may be omitted too. Create variables to
    // store the return values
    for (i, p) in def.returns.iter().enumerate() {
        let ret = &ns.functions[function_no].returns[i];

        if let Some(ref name) = p.1.as_ref().unwrap().name {
            return_required = false;

            if let Some(pos) = symtable.add(
                name,
                ret.ty.clone(),
                ns,
                VariableInitializer::Solidity(None),
                VariableUsage::ReturnVariable,
            ) {
                ns.check_shadowing(file_no, contract_no, name);
                symtable.returns.push(pos);
            }
        } else {
            // anonymous return
            let id = program::Identifier {
                loc: p.0,
                name: "".to_owned(),
            };

            let pos = symtable
                .add(
                    &id,
                    ret.ty.clone(),
                    ns,
                    VariableInitializer::Solidity(None),
                    VariableUsage::AnonymousReturnVariable,
                    None,
                )
                .unwrap();

            symtable.returns.push(pos);
        }
    }

    let body = match def.body {
        None => return Ok(()),
        Some(ref body) => body,
    };

    let mut diagnostics = Diagnostics::default();

    let reachable = statement(
        body,
        &mut res,
        &context,
        &mut symtable,
        &mut loops,
        ns,
        &mut diagnostics,
    );

    ns.diagnostics.extend(diagnostics);

    if reachable? && return_required {
        ns.diagnostics.push(Diagnostic::error(
            body.loc().end_range(),
            "missing return statement".to_string(),
        ));
        return Err(());
    }

    if def.ty == program::FunctionTy::Modifier {
        let mut has_underscore = false;

        // unsure modifier has underscore
        fn check_statement(stmt: &Statement, has_underscore: &mut bool) -> bool {
            if stmt.is_underscore() {
                *has_underscore = true;
                false
            } else {
                true
            }
        }

        for stmt in &mut res {
            stmt.recurse(&mut has_underscore, check_statement);
        }

        if !has_underscore {
            ns.diagnostics.push(Diagnostic::error(
                body.loc().end_range(),
                "missing '_' in modifier".to_string(),
            ));
        }
    }

    ns.functions[function_no].body = res;

    std::mem::swap(&mut ns.functions[function_no].symtable, &mut symtable);

    Ok(())
}

/// Resolve a statement
#[allow(clippy::ptr_arg)]
fn statement(
    stmt: &program::Statement,
    res: &mut Vec<Statement>,
    context: &ExprContext,
    symtable: &mut Symtable,
    loops: &mut LoopScopes,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<bool, ()> {
    let function_no = context.function_no.unwrap();

    match stmt {
        program::Statement::VariableDefinition(loc, decl, initializer) => {
            let (var_ty, ty_loc) =
                resolve_var_decl_ty(&decl.ty, &decl.storage, context, ns, diagnostics)?;

            let initializer = if let Some(init) = initializer {
                let expr = expression(
                    init,
                    context,
                    ns,
                    symtable,
                    diagnostics,
                    ResolveTo::Type(&var_ty),
                )?;

                expr.recurse(ns, check_term_for_constant_overflow);
                used_variable(ns, &expr, symtable);

                Some(Arc::new(expr.cast(
                    &expr.loc(),
                    &var_ty,
                    true,
                    ns,
                    diagnostics,
                )?))
            } else {
                None
            };

            if let Some(pos) = symtable.add(
                decl.name.as_ref().unwrap(),
                var_ty.clone(),
                ns,
                VariableInitializer::Solidity(initializer.clone()),
                VariableUsage::LocalVariable,
                decl.storage.clone(),
            ) {
                ns.check_shadowing(
                    context.file_no,
                    context.contract_no,
                    decl.name.as_ref().unwrap(),
                );

                res.push(Statement::VariableDecl(
                    *loc,
                    pos,
                    Parameter {
                        loc: decl.loc,
                        ty: var_ty,
                        ty_loc: Some(ty_loc),
                        id: Some(decl.name.clone().unwrap()),
                        recursive: false,
                    },
                    initializer,
                ));
            }

            Ok(true)
        }
        program::Statement::Block {
            statements,
            unchecked,
            ..
        } => {
            symtable.new_scope();
            let mut reachable = true;

            let mut context = context.clone();
            context.unchecked |= *unchecked;

            for stmt in statements {
                if !reachable {
                    ns.diagnostics.push(Diagnostic::error(
                        stmt.loc(),
                        "unreachable statement".to_string(),
                    ));
                    return Err(());
                }
                reachable = statement(stmt, res, &context, symtable, loops, ns, diagnostics)?;
            }

            symtable.leave_scope();

            Ok(reachable)
        }
        program::Statement::Break(loc) => {
            if loops.do_break() {
                res.push(Statement::Break(*loc));
                Ok(false)
            } else {
                diagnostics.push(Diagnostic::error(
                    stmt.loc(),
                    "break statement not in loop".to_string(),
                ));
                Err(())
            }
        }
        program::Statement::Continue(loc) => {
            if loops.do_continue() {
                res.push(Statement::Continue(*loc));
                Ok(false)
            } else {
                diagnostics.push(Diagnostic::error(
                    stmt.loc(),
                    "continue statement not in loop".to_string(),
                ));
                Err(())
            }
        }
        program::Statement::While(loc, cond_expr, body) => {
            let expr = expression(
                cond_expr,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;
            used_variable(ns, &expr, symtable);
            let cond = expr.cast(&expr.loc(), &Type::Bool, true, ns, diagnostics)?;

            symtable.new_scope();
            let mut body_stmts = Vec::new();
            loops.new_scope();
            statement(
                body,
                &mut body_stmts,
                context,
                symtable,
                loops,
                ns,
                diagnostics,
            )?;
            symtable.leave_scope();
            loops.leave_scope();

            res.push(Statement::While(*loc, true, cond, body_stmts));

            Ok(true)
        }
        program::Statement::DoWhile(loc, body, cond_expr) => {
            let expr = expression(
                cond_expr,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;
            used_variable(ns, &expr, symtable);
            let cond = expr.cast(&expr.loc(), &Type::Bool, true, ns, diagnostics)?;

            symtable.new_scope();
            let mut body_stmts = Vec::new();
            loops.new_scope();
            statement(
                body,
                &mut body_stmts,
                context,
                symtable,
                loops,
                ns,
                diagnostics,
            )?;
            symtable.leave_scope();
            loops.leave_scope();

            res.push(Statement::DoWhile(*loc, true, body_stmts, cond));
            Ok(true)
        }
        program::Statement::If(loc, cond_expr, then, else_) => {
            let expr = expression(
                cond_expr,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;
            used_variable(ns, &expr, symtable);

            let cond = expr.cast(&expr.loc(), &Type::Bool, true, ns, diagnostics)?;

            symtable.new_scope();
            let mut then_stmts = Vec::new();
            let mut reachable = statement(
                then,
                &mut then_stmts,
                context,
                symtable,
                loops,
                ns,
                diagnostics,
            )?;
            symtable.leave_scope();

            let mut else_stmts = Vec::new();
            if let Some(stmts) = else_ {
                symtable.new_scope();
                reachable |= statement(
                    stmts,
                    &mut else_stmts,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;

                symtable.leave_scope();
            } else {
                reachable = true;
            }

            res.push(Statement::If(*loc, reachable, cond, then_stmts, else_stmts));

            Ok(reachable)
        }
        program::Statement::Args(loc, _) => {
            ns.diagnostics.push(Diagnostic::error(
                *loc,
                "expected code block, not list of named arguments".to_string(),
            ));
            Err(())
        }
        program::Statement::For(loc, init_stmt, None, next_stmt, body_stmt) => {
            symtable.new_scope();

            let mut init = Vec::new();

            if let Some(init_stmt) = init_stmt {
                statement(
                    init_stmt,
                    &mut init,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;
            }

            loops.new_scope();

            let mut body = Vec::new();

            if let Some(body_stmt) = body_stmt {
                statement(
                    body_stmt,
                    &mut body,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;
            }

            let control = loops.leave_scope();
            let reachable = control.no_breaks > 0;
            let mut next = Vec::new();

            if let Some(next_stmt) = next_stmt {
                statement(
                    next_stmt,
                    &mut next,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;
            }

            symtable.leave_scope();

            res.push(Statement::For {
                loc: *loc,
                reachable,
                init,
                next,
                cond: None,
                body,
            });

            Ok(reachable)
        }
        program::Statement::For(loc, init_stmt, Some(cond_expr), next_stmt, body_stmt) => {
            symtable.new_scope();

            let mut init = Vec::new();
            let mut body = Vec::new();
            let mut next = Vec::new();

            if let Some(init_stmt) = init_stmt {
                statement(
                    init_stmt,
                    &mut init,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;
            }

            let expr = expression(
                cond_expr,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;

            let cond = expr.cast(&cond_expr.loc(), &Type::Bool, true, ns, diagnostics)?;

            // continue goes to next, and if that does exist, cond
            loops.new_scope();

            let mut body_reachable = match body_stmt {
                Some(body_stmt) => statement(
                    body_stmt,
                    &mut body,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?,
                None => true,
            };

            let control = loops.leave_scope();

            if control.no_continues > 0 {
                body_reachable = true;
            }

            if body_reachable {
                if let Some(next_stmt) = next_stmt {
                    statement(
                        next_stmt,
                        &mut next,
                        context,
                        symtable,
                        loops,
                        ns,
                        diagnostics,
                    )?;
                }
            }

            symtable.leave_scope();

            res.push(Statement::For {
                loc: *loc,
                reachable: true,
                init,
                next,
                cond: Some(cond),
                body,
            });

            Ok(true)
        }
        program::Statement::Return(loc, None) => {
            let no_returns = ns.functions[context.function_no.unwrap()].returns.len();

            if symtable.returns.len() != no_returns {
                ns.diagnostics.push(Diagnostic::error(
                    *loc,
                    format!(
                        "missing return value, {} return values expected",
                        no_returns
                    ),
                ));
                return Err(());
            }

            res.push(Statement::Return(*loc, None));

            Ok(false)
        }
        program::Statement::Return(loc, Some(returns)) => {
            let expr = return_with_values(returns, loc, context, symtable, ns, diagnostics)?;

            expr.recurse(ns, check_term_for_constant_overflow);

            for offset in symtable.returns.iter() {
                let elem = symtable.vars.get_mut(offset).unwrap();
                elem.assigned = true;
            }

            res.push(Statement::Return(*loc, Some(expr)));

            Ok(false)
        }
        program::Statement::Expression(loc, expr) => {
            let expr = match expr {
                // delete statement
                program::Expression::Delete(_, expr) => {
                    let expr =
                        expression(expr, context, ns, symtable, diagnostics, ResolveTo::Unknown)?;
                    used_variable(ns, &expr, symtable);
                    return if let Type::StorageRef(_, ty) = expr.ty() {
                        if expr.ty().is_mapping() {
                            ns.diagnostics.push(Diagnostic::error(
                                *loc,
                                "'delete' cannot be applied to mapping type".to_string(),
                            ));
                            return Err(());
                        }

                        res.push(Statement::Delete(*loc, ty.as_ref().clone(), expr));

                        Ok(true)
                    } else {
                        ns.diagnostics.push(Diagnostic::error(
                            *loc,
                            "argument to 'delete' should be storage reference".to_string(),
                        ));

                        Err(())
                    };
                }
                // is it an underscore modifier statement
                program::Expression::Variable(id) if id.name == "_" => {
                    return if ns.functions[function_no].ty == program::FunctionTy::Modifier {
                        res.push(Statement::Underscore(*loc));
                        Ok(true)
                    } else {
                        ns.diagnostics.push(Diagnostic::error(
                            *loc,
                            "underscore statement only permitted in modifiers".to_string(),
                        ));
                        Err(())
                    };
                }
                program::Expression::FunctionCall(loc, ty, args) => {
                    let ret = call_expr(
                        loc,
                        ty,
                        args,
                        true,
                        context,
                        ns,
                        symtable,
                        diagnostics,
                        ResolveTo::Discard,
                    )?;

                    ret.recurse(ns, check_term_for_constant_overflow);
                    ret
                }
                program::Expression::NamedFunctionCall(loc, ty, args) => {
                    let ret = named_call_expr(
                        loc,
                        ty,
                        args,
                        true,
                        context,
                        ns,
                        symtable,
                        diagnostics,
                        ResolveTo::Discard,
                    )?;
                    ret.recurse(ns, check_term_for_constant_overflow);
                    ret
                }
                _ => {
                    // is it a destructure statement
                    if let program::Expression::Assign(_, var, expr) = expr {
                        if let program::Expression::List(_, var) = var.as_ref() {
                            res.push(destructure(
                                loc,
                                var,
                                expr,
                                context,
                                symtable,
                                ns,
                                diagnostics,
                            )?);

                            // if a noreturn function was called, then the destructure would not resolve
                            return Ok(true);
                        }
                    }
                    // the rest. We don't care about the result
                    expression(expr, context, ns, symtable, diagnostics, ResolveTo::Unknown)?
                }
            };

            let reachable = expr.tys() != vec![Type::Unreachable];

            res.push(Statement::Expression(*loc, reachable, expr));

            Ok(reachable)
        }
        program::Statement::Try(loc, expr, returns_and_ok, clause_stmts) => {
            let (stmt, reachable) = try_catch(
                loc,
                expr,
                returns_and_ok,
                clause_stmts,
                context,
                symtable,
                loops,
                ns,
                diagnostics,
            )?;
            res.push(stmt);

            Ok(reachable)
        }
        program::Statement::Emit(loc, ty) => {
            if let Ok(emit) = emit_event(loc, ty, context, symtable, ns, diagnostics) {
                res.push(emit);
            }

            Ok(true)
        }
        program::Statement::Assembly {
            loc,
            dialect,
            flags,
            block,
        } => {
            if dialect.is_some() && dialect.as_ref().unwrap().string != "evmasm" {
                ns.diagnostics.push(Diagnostic::error(
                    dialect.as_ref().unwrap().loc,
                    "only evmasm dialect is supported".to_string(),
                ));
                return Err(());
            }

            if let Some(flags) = flags {
                for flag in flags {
                    ns.diagnostics.push(Diagnostic::error(
                        flag.loc,
                        format!("flag '{}' not supported", flag.string),
                    ));
                }
            }

            let resolved_asm =
                resolve_inline_assembly(loc, &block.statements, context, symtable, ns);
            res.push(Statement::Assembly(resolved_asm.0, resolved_asm.1));
            Ok(resolved_asm.1)
        }
        program::Statement::Revert(loc, error, args) => {
            if let Some(error) = error {
                ns.diagnostics.push(Diagnostic::error(
                    error.loc,
                    format!("revert with custom error '{}' not supported yet", error),
                ));
                return Err(());
            }

            let id = program::Identifier {
                loc: program::Loc::File(loc.file_no(), loc.start(), loc.start() + 6),
                name: "revert".to_string(),
            };

            let expr = builtin::resolve_call(
                &id.loc,
                None,
                &id.name,
                args,
                context,
                ns,
                symtable,
                diagnostics,
            )?;

            let reachable = expr.ty() != Type::Unreachable;

            res.push(Statement::Expression(*loc, reachable, expr));

            Ok(reachable)
        }
        program::Statement::RevertNamedArgs(loc, _, _) => {
            ns.diagnostics.push(Diagnostic::error(
                *loc,
                "revert with custom errors or named arguments not supported yet".to_string(),
            ));
            Err(())
        }
        program::Statement::Error(_) => unimplemented!(),
    }
}

/// Resolve emit event
fn emit_event(
    loc: &program::Loc,
    ty: &program::Expression,
    context: &ExprContext,
    symtable: &mut Symtable,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<Statement, ()> {
    let function_no = context.function_no.unwrap();

    match ty {
        program::Expression::FunctionCall(_, ty, args) => {
            let event_loc = ty.loc();

            let mut errors = Diagnostics::default();

            let event_nos =
                match ns.resolve_event(context.file_no, context.contract_no, ty, diagnostics) {
                    Ok(nos) => nos,
                    Err(_) => {
                        for arg in args {
                            if let Ok(exp) = expression(
                                arg,
                                context,
                                ns,
                                symtable,
                                diagnostics,
                                ResolveTo::Unknown,
                            ) {
                                used_variable(ns, &exp, symtable);
                            };
                        }
                        return Err(());
                    }
                };

            for event_no in &event_nos {
                let event = &mut ns.events[*event_no];
                event.used = true;

                let mut matches = true;

                if args.len() != event.fields.len() {
                    errors.push(Diagnostic::cast_error(
                        *loc,
                        format!(
                            "event type '{}' has {} fields, {} provided",
                            event.name,
                            event.fields.len(),
                            args.len()
                        ),
                    ));
                    matches = false;
                }
                let mut cast_args = Vec::new();

                // check if arguments can be implicitly casted
                for (i, arg) in args.iter().enumerate() {
                    let ty = ns.events[*event_no]
                        .fields
                        .get(i)
                        .map(|field| field.ty.clone());

                    let resolve_to = ty
                        .as_ref()
                        .map(ResolveTo::Type)
                        .unwrap_or(ResolveTo::Unknown);

                    let arg = match expression(arg, context, ns, symtable, &mut errors, resolve_to)
                    {
                        Ok(e) => e,
                        Err(()) => {
                            matches = false;
                            break;
                        }
                    };
                    used_variable(ns, &arg, symtable);

                    if let Some(ty) = &ty {
                        match arg.cast(&arg.loc(), ty, true, ns, &mut errors) {
                            Ok(expr) => cast_args.push(expr),
                            Err(_) => {
                                matches = false;
                            }
                        }
                    }
                }

                if matches {
                    if !ns.functions[function_no].emits_events.contains(event_no) {
                        ns.functions[function_no].emits_events.push(*event_no);
                    }

                    return Ok(Statement::Emit {
                        loc: *loc,
                        event_no: *event_no,
                        event_loc,
                        args: cast_args,
                    });
                } else if event_nos.len() > 1 && diagnostics.extend_non_casting(&errors) {
                    return Err(());
                }
            }

            if event_nos.len() == 1 {
                diagnostics.extend(errors);
            } else {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    "cannot find event which matches signature".to_string(),
                ));
            }
        }
        program::Expression::NamedFunctionCall(_, ty, args) => {
            let event_loc = ty.loc();

            let mut temp_diagnostics = Diagnostics::default();
            let mut arguments = HashMap::new();

            for arg in args {
                if arguments.contains_key(arg.name.name.as_str()) {
                    diagnostics.push(Diagnostic::error(
                        arg.name.loc,
                        format!("duplicate argument with name '{}'", arg.name.name),
                    ));

                    let _ = expression(
                        &arg.expr,
                        context,
                        ns,
                        symtable,
                        diagnostics,
                        ResolveTo::Unknown,
                    );

                    continue;
                }

                arguments.insert(arg.name.name.as_str(), &arg.expr);
            }

            let event_nos = match ns.resolve_event(
                context.file_no,
                context.contract_no,
                ty,
                &mut temp_diagnostics,
            ) {
                Ok(nos) => nos,
                Err(_) => {
                    // check arguments for errors
                    for (_, arg) in arguments {
                        let _ =
                            expression(arg, context, ns, symtable, diagnostics, ResolveTo::Unknown);
                    }
                    return Err(());
                }
            };

            for event_no in &event_nos {
                let event = &mut ns.events[*event_no];
                event.used = true;
                let params_len = event.fields.len();

                let mut matches = true;

                let unnamed_fields = event.fields.iter().filter(|p| p.id.is_none()).count();

                if unnamed_fields > 0 {
                    temp_diagnostics.push(Diagnostic::cast_error_with_note(
                        *loc,
                        format!(
                            "event cannot be emmited with named fields as {} of its fields do not have names",
                            unnamed_fields,
                        ),
                        event.loc,
                        format!("definition of {}", event.name),
                    ));
                    matches = false;
                } else if params_len != arguments.len() {
                    temp_diagnostics.push(Diagnostic::error(
                        *loc,
                        format!(
                            "event expects {} arguments, {} provided",
                            params_len,
                            arguments.len()
                        ),
                    ));
                    matches = false;
                }

                let mut cast_args = Vec::new();

                // check if arguments can be implicitly casted
                for i in 0..params_len {
                    let param = ns.events[*event_no].fields[i].clone();

                    if param.id.is_none() {
                        continue;
                    }

                    let arg = match arguments.get(param.name_as_str()) {
                        Some(a) => a,
                        None => {
                            matches = false;
                            temp_diagnostics.push(Diagnostic::cast_error(
                                *loc,
                                format!(
                                    "missing argument '{}' to event '{}'",
                                    param.name_as_str(),
                                    ns.events[*event_no].name,
                                ),
                            ));
                            continue;
                        }
                    };

                    let arg = match expression(
                        arg,
                        context,
                        ns,
                        symtable,
                        &mut temp_diagnostics,
                        ResolveTo::Type(&param.ty),
                    ) {
                        Ok(e) => e,
                        Err(()) => {
                            matches = false;
                            continue;
                        }
                    };

                    used_variable(ns, &arg, symtable);

                    match arg.cast(&arg.loc(), &param.ty, true, ns, &mut temp_diagnostics) {
                        Ok(expr) => cast_args.push(expr),
                        Err(_) => {
                            matches = false;
                        }
                    }
                }

                if matches {
                    if !ns.functions[function_no].emits_events.contains(event_no) {
                        ns.functions[function_no].emits_events.push(*event_no);
                    }

                    return Ok(Statement::Emit {
                        loc: *loc,
                        event_no: *event_no,
                        event_loc,
                        args: cast_args,
                    });
                } else if event_nos.len() > 1 && diagnostics.extend_non_casting(&temp_diagnostics) {
                    return Err(());
                }
            }

            if event_nos.len() == 1 {
                diagnostics.extend(temp_diagnostics);
            } else {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    "cannot find event which matches signature".to_string(),
                ));
            }
        }
        program::Expression::FunctionCallBlock(_, ty, block) => {
            let _ = ns.resolve_event(context.file_no, context.contract_no, ty, diagnostics);

            diagnostics.push(Diagnostic::error(
                block.loc(),
                "expected event arguments, found code block".to_string(),
            ));
        }
        _ => unreachable!(),
    }

    Err(())
}

/// Resolve destructuring assignment
fn destructure(
    loc: &program::Loc,
    vars: &[(program::Loc, Option<program::Parameter>)],
    expr: &program::Expression,
    context: &ExprContext,
    symtable: &mut Symtable,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<Statement, ()> {
    // first resolve the fields so we know the types
    let mut fields = Vec::new();
    let mut left_tys = Vec::new();

    let mut lcontext = context.clone();
    lcontext.lvalue = true;

    for (_, param) in vars {
        match param {
            None => {
                left_tys.push(None);
                fields.push(DestructureField::None);
            }
            Some(program::Parameter {
                loc,
                ty,
                storage,
                name: None,
            }) => {
                if let Some(storage) = storage {
                    diagnostics.push(Diagnostic::error(
                        storage.loc(),
                        format!("storage modifier '{}' not permitted on assignment", storage),
                    ));
                    return Err(());
                }

                // ty will just be a normal expression, not a type
                let e = expression(ty, &lcontext, ns, symtable, diagnostics, ResolveTo::Unknown)?;

                match &e {
                    Expression::ConstantVariable(_, _, Some(contract_no), var_no) => {
                        diagnostics.push(Diagnostic::error(
                            *loc,
                            format!(
                                "cannot assign to constant '{}'",
                                ns.contracts[*contract_no].variables[*var_no].name
                            ),
                        ));
                        return Err(());
                    }
                    Expression::ConstantVariable(_, _, None, var_no) => {
                        diagnostics.push(Diagnostic::error(
                            *loc,
                            format!("cannot assign to constant '{}'", ns.constants[*var_no].name),
                        ));
                        return Err(());
                    }
                    Expression::StorageVariable(_, _, var_contract_no, var_no) => {
                        let store_var = &ns.contracts[*var_contract_no].variables[*var_no];

                        if store_var.immutable
                            && !ns.functions[context.function_no.unwrap()].is_constructor()
                        {
                            diagnostics.push(Diagnostic::error(
                                *loc,
                                format!(
                                    "cannot assign to immutable '{}' outside of constructor",
                                    store_var.name
                                ),
                            ));
                            return Err(());
                        }
                    }
                    Expression::Variable(..) => (),
                    _ => match e.ty() {
                        Type::Ref(_) | Type::StorageRef(false, _) => (),
                        _ => {
                            diagnostics.push(Diagnostic::error(
                                *loc,
                                "expression is not assignable".to_string(),
                            ));
                            return Err(());
                        }
                    },
                }

                assigned_variable(ns, &e, symtable);
                left_tys.push(Some(e.ty()));
                fields.push(DestructureField::Expression(e));
            }
            Some(program::Parameter {
                loc,
                ty,
                storage,
                name: Some(name),
            }) => {
                let (ty, ty_loc) = resolve_var_decl_ty(ty, storage, context, ns, diagnostics)?;

                if let Some(pos) = symtable.add(
                    name,
                    ty.clone(),
                    ns,
                    VariableInitializer::Solidity(None),
                    VariableUsage::DestructureVariable,
                    storage.clone(),
                ) {
                    ns.check_shadowing(context.file_no, context.contract_no, name);

                    left_tys.push(Some(ty.clone()));

                    fields.push(DestructureField::VariableDecl(
                        pos,
                        Parameter {
                            loc: *loc,
                            id: Some(name.clone()),
                            ty,
                            ty_loc: Some(ty_loc),
                            recursive: false,
                        },
                    ));
                }
            }
        }
    }

    let expr = destructure_values(
        loc,
        expr,
        &left_tys,
        &fields,
        context,
        symtable,
        ns,
        diagnostics,
    )?;

    Ok(Statement::Destructure(*loc, fields, expr))
}

fn destructure_values(
    loc: &program::Loc,
    expr: &program::Expression,
    left_tys: &[Option<Type>],
    fields: &[DestructureField],
    context: &ExprContext,
    symtable: &mut Symtable,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<Expression, ()> {
    let expr = match expr.remove_parenthesis() {
        program::Expression::FunctionCall(loc, ty, args) => {
            let res = function_call_expr(
                loc,
                ty,
                args,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Unknown,
            )?;
            check_function_call(ns, &res, symtable);
            res
        }
        program::Expression::NamedFunctionCall(loc, ty, args) => {
            let res = named_function_call_expr(
                loc,
                ty,
                args,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Unknown,
            )?;
            check_function_call(ns, &res, symtable);
            res
        }
        program::Expression::ConditionalOperator(loc, cond, left, right) => {
            let cond = expression(
                cond,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;

            used_variable(ns, &cond, symtable);
            let left = destructure_values(
                &left.loc(),
                left,
                left_tys,
                fields,
                context,
                symtable,
                ns,
                diagnostics,
            )?;
            used_variable(ns, &left, symtable);
            let right = destructure_values(
                &right.loc(),
                right,
                left_tys,
                fields,
                context,
                symtable,
                ns,
                diagnostics,
            )?;
            used_variable(ns, &right, symtable);

            return Ok(Expression::ConditionalOperator(
                *loc,
                Type::Unreachable,
                Box::new(cond),
                Box::new(left),
                Box::new(right),
            ));
        }
        _ => {
            let mut list = Vec::new();

            let exprs = parameter_list_to_expr_list(expr, diagnostics)?;

            if exprs.len() != left_tys.len() {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    format!(
                        "destructuring assignment has {} elements on the left and {} on the right",
                        left_tys.len(),
                        exprs.len(),
                    ),
                ));
                return Err(());
            }

            for (i, e) in exprs.iter().enumerate() {
                let e = expression(
                    e,
                    context,
                    ns,
                    symtable,
                    diagnostics,
                    if let Some(ty) = left_tys[i].as_ref() {
                        ResolveTo::Type(ty)
                    } else {
                        ResolveTo::Unknown
                    },
                )?;
                match e.ty() {
                    Type::Void | Type::Unreachable => {
                        diagnostics.push(Diagnostic::error(
                            e.loc(),
                            "function does not return a value".to_string(),
                        ));
                        return Err(());
                    }
                    _ => {
                        used_variable(ns, &e, symtable);
                    }
                }

                list.push(e);
            }

            Expression::List(*loc, list)
        }
    };

    let mut right_tys = expr.tys();

    // Return type void or unreachable are synthetic
    if right_tys.len() == 1 && (right_tys[0] == Type::Unreachable || right_tys[0] == Type::Void) {
        right_tys.truncate(0);
    }

    if left_tys.len() != right_tys.len() {
        diagnostics.push(Diagnostic::error(
            *loc,
            format!(
                "destructuring assignment has {} elements on the left and {} on the right",
                left_tys.len(),
                right_tys.len()
            ),
        ));
        return Err(());
    }

    // Check that the values can be cast
    for (i, field) in fields.iter().enumerate() {
        if let Some(left_ty) = &left_tys[i] {
            let loc = field.loc().unwrap();
            let _ = Expression::Variable(loc, right_tys[i].clone(), i).cast(
                &loc,
                left_ty.deref_memory(),
                true,
                ns,
                diagnostics,
            )?;
        }
    }
    Ok(expr)
}

/// Resolve the type of a variable declaration
fn resolve_var_decl_ty(
    ty: &program::Expression,
    storage: &Option<program::StorageLocation>,
    context: &ExprContext,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<(Type, program::Loc), ()> {
    let mut loc_ty = ty.loc();
    let mut var_ty =
        ns.resolve_type(context.file_no, context.contract_no, false, ty, diagnostics)?;

    if let Some(storage) = storage {
        if !var_ty.can_have_data_location() {
            diagnostics.push(Diagnostic::error(
                storage.loc(),
                format!(
                    "data location '{}' only allowed for array, struct or mapping type",
                    storage
                ),
            ));
            return Err(());
        }

        if let program::StorageLocation::Storage(loc) = storage {
            loc_ty.use_end_from(loc);
            var_ty = Type::StorageRef(false, Box::new(var_ty));
        }

        // Note we are completely ignoring memory or calldata data locations. Everything
        // will be stored in memory.
    }

    if var_ty.contains_mapping(ns) && !var_ty.is_contract_storage() {
        diagnostics.push(Diagnostic::error(
            ty.loc(),
            "mapping only allowed in storage".to_string(),
        ));
        return Err(());
    }

    if !var_ty.is_contract_storage() && !var_ty.fits_in_memory(ns) {
        diagnostics.push(Diagnostic::error(
            ty.loc(),
            "type is too large to fit into memory".to_string(),
        ));
        return Err(());
    }

    Ok((var_ty, loc_ty))
}

/// Resolve return statement
fn return_with_values(
    returns: &program::Expression,
    loc: &program::Loc,
    context: &ExprContext,
    symtable: &mut Symtable,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<Expression, ()> {
    let function_no = context.function_no.unwrap();

    let no_returns = ns.functions[function_no].returns.len();
    let expr_returns = match returns.remove_parenthesis() {
        program::Expression::FunctionCall(loc, ty, args) => {
            let expr = call_expr(
                loc,
                ty,
                args,
                true,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Unknown,
            )?;
            used_variable(ns, &expr, symtable);
            expr
        }
        program::Expression::NamedFunctionCall(loc, ty, args) => {
            let expr = named_call_expr(
                loc,
                ty,
                args,
                true,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Unknown,
            )?;
            used_variable(ns, &expr, symtable);
            expr
        }
        program::Expression::ConditionalOperator(loc, cond, left, right) => {
            let cond = expression(
                cond,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Type(&Type::Bool),
            )?;
            used_variable(ns, &cond, symtable);

            let left = return_with_values(left, &left.loc(), context, symtable, ns, diagnostics)?;
            used_variable(ns, &left, symtable);

            let right =
                return_with_values(right, &right.loc(), context, symtable, ns, diagnostics)?;
            used_variable(ns, &right, symtable);

            return Ok(Expression::ConditionalOperator(
                *loc,
                Type::Unreachable,
                Box::new(cond),
                Box::new(left),
                Box::new(right),
            ));
        }
        _ => {
            let returns = parameter_list_to_expr_list(returns, diagnostics)?;

            if no_returns > 0 && returns.is_empty() {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    format!(
                        "missing return value, {} return values expected",
                        no_returns
                    ),
                ));
                return Err(());
            }

            if no_returns == 0 && !returns.is_empty() {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    "function has no return values".to_string(),
                ));
                return Err(());
            }

            if no_returns != returns.len() {
                diagnostics.push(Diagnostic::error(
                    *loc,
                    format!(
                        "incorrect number of return values, expected {} but got {}",
                        no_returns,
                        returns.len(),
                    ),
                ));
                return Err(());
            }

            let mut exprs = Vec::new();

            let return_tys = ns.functions[function_no]
                .returns
                .iter()
                .map(|r| r.ty.clone())
                .collect::<Vec<_>>();

            for (expr_return, return_ty) in returns.iter().zip(return_tys) {
                let expr = expression(
                    expr_return,
                    context,
                    ns,
                    symtable,
                    diagnostics,
                    ResolveTo::Type(&return_ty),
                )?;
                let expr = expr.cast(loc, &return_ty, true, ns, diagnostics)?;
                used_variable(ns, &expr, symtable);
                exprs.push(expr);
            }

            return Ok(if exprs.len() == 1 {
                exprs[0].clone()
            } else {
                Expression::List(*loc, exprs)
            });
        }
    };

    let mut expr_return_tys = expr_returns.tys();
    // Return type void or unreachable are synthetic
    if expr_return_tys.len() == 1
        && (expr_return_tys[0] == Type::Unreachable || expr_return_tys[0] == Type::Void)
    {
        expr_return_tys.truncate(0);
    }

    if no_returns > 0 && expr_return_tys.is_empty() {
        diagnostics.push(Diagnostic::error(
            *loc,
            format!(
                "missing return value, {} return values expected",
                no_returns
            ),
        ));
        return Err(());
    }

    if no_returns == 0 && !expr_return_tys.is_empty() {
        diagnostics.push(Diagnostic::error(
            *loc,
            "function has no return values".to_string(),
        ));
        return Err(());
    }

    if no_returns != expr_return_tys.len() {
        diagnostics.push(Diagnostic::error(
            *loc,
            format!(
                "incorrect number of return values, expected {} but got {}",
                no_returns,
                expr_return_tys.len(),
            ),
        ));
        return Err(());
    }

    let func_returns_tys = ns.functions[function_no]
        .returns
        .iter()
        .map(|r| r.ty.clone())
        .collect::<Vec<_>>();

    // Check that the values can be cast
    let _ = expr_return_tys
        .into_iter()
        .zip(func_returns_tys)
        .enumerate()
        .map(|(i, (expr_return_ty, func_return_ty))| {
            Expression::Variable(expr_returns.loc(), expr_return_ty, i).cast(
                &expr_returns.loc(),
                &func_return_ty,
                true,
                ns,
                diagnostics,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(expr_returns)
}

/// The parser generates parameter lists for lists. Sometimes this needs to be a
/// simple expression list.
pub fn parameter_list_to_expr_list<'a>(
    e: &'a program::Expression,
    diagnostics: &mut Diagnostics,
) -> Result<Vec<&'a program::Expression>, ()> {
    match e {
        program::Expression::List(_, v) => {
            let mut list = Vec::new();
            let mut broken = false;

            for e in v {
                match &e.1 {
                    None => {
                        diagnostics.push(Diagnostic::error(e.0, "stray comma".to_string()));
                        broken = true;
                    }
                    Some(program::Parameter {
                        name: Some(name), ..
                    }) => {
                        diagnostics.push(Diagnostic::error(
                            name.loc,
                            "single value expected".to_string(),
                        ));
                        broken = true;
                    }
                    Some(program::Parameter {
                        storage: Some(storage),
                        ..
                    }) => {
                        diagnostics.push(Diagnostic::error(
                            storage.loc(),
                            "storage specified not permitted here".to_string(),
                        ));
                        broken = true;
                    }
                    Some(program::Parameter { ty, .. }) => {
                        list.push(ty);
                    }
                }
            }

            if !broken {
                Ok(list)
            } else {
                Err(())
            }
        }
        program::Expression::Parenthesis(_, e) => Ok(vec![e]),
        e => Ok(vec![e]),
    }
}

/// Parse try catch
#[allow(clippy::type_complexity)]
fn try_catch(
    loc: &program::Loc,
    expr: &program::Expression,
    returns_and_ok: &Option<(Vec<(program::Loc, Option<program::Parameter>)>, Box<program::Statement>)>,
    clause_stmts: &[program::CatchClause],
    context: &ExprContext,
    symtable: &mut Symtable,
    loops: &mut LoopScopes,
    ns: &mut Namespace,
    diagnostics: &mut Diagnostics,
) -> Result<(Statement, bool), ()> {
    if ns.target == Target::Solana {
        diagnostics.push(Diagnostic::error(
            *loc,
            "The try-catch statement is not supported on Solana. Please, go to \
             https://solang.readthedocs.io/en/latest/language/statements.html#try-catch-statement \
             for more information"
                .to_string(),
        ));
        return Err(());
    }

    let mut expr = expr.remove_parenthesis();
    let mut ok = None;

    while let program::Expression::FunctionCallBlock(_, e, block) = expr {
        if ok.is_some() {
            diagnostics.push(Diagnostic::error(
                block.loc(),
                "unexpected code block".to_string(),
            ));
            return Err(());
        }

        ok = Some(block.as_ref());

        expr = e.as_ref();
    }

    let fcall = match expr.remove_parenthesis() {
        program::Expression::FunctionCall(loc, ty, args) => {
            let res = match ty.remove_parenthesis() {
                program::Expression::New(_, ty) => {
                    new(loc, ty, args, context, ns, symtable, diagnostics)?
                }
                program::Expression::FunctionCallBlock(loc, expr, _)
                    if matches!(expr.remove_parenthesis(), program::Expression::New(..)) =>
                {
                    new(loc, ty, args, context, ns, symtable, diagnostics)?
                }
                _ => function_call_expr(
                    loc,
                    ty,
                    args,
                    context,
                    ns,
                    symtable,
                    diagnostics,
                    ResolveTo::Unknown,
                )?,
            };
            check_function_call(ns, &res, symtable);
            res
        }
        program::Expression::NamedFunctionCall(loc, ty, args) => {
            let res = named_function_call_expr(
                loc,
                ty,
                args,
                context,
                ns,
                symtable,
                diagnostics,
                ResolveTo::Unknown,
            )?;

            check_function_call(ns, &res, symtable);

            res
        }
        program::Expression::New(loc, call) => {
            let mut call = call.remove_parenthesis();

            while let program::Expression::FunctionCallBlock(_, expr, block) = call {
                if ok.is_some() {
                    ns.diagnostics.push(Diagnostic::error(
                        block.loc(),
                        "unexpected code block".to_string(),
                    ));
                    return Err(());
                }

                ok = Some(block.as_ref());

                call = expr.remove_parenthesis();
            }

            match call {
                program::Expression::FunctionCall(_, ty, args) => {
                    let res = new(loc, ty, args, context, ns, symtable, diagnostics)?;
                    check_function_call(ns, &res, symtable);

                    res
                }
                program::Expression::NamedFunctionCall(_, ty, args) => {
                    let res =
                        constructor_named_args(loc, ty, args, context, ns, symtable, diagnostics)?;
                    check_function_call(ns, &res, symtable);

                    res
                }
                _ => unreachable!(),
            }
        }
        _ => {
            diagnostics.push(Diagnostic::error(
                expr.loc(),
                "try only supports external calls or constructor calls".to_string(),
            ));
            return Err(());
        }
    };

    let mut returns = &Vec::new();

    if let Some((rets, block)) = returns_and_ok {
        if ok.is_some() {
            diagnostics.push(Diagnostic::error(
                block.loc(),
                "unexpected code block".to_string(),
            ));
            return Err(());
        }

        ok = Some(block);

        returns = rets;
    }

    let ok = match ok {
        Some(ok) => ok,
        None => {
            // position after the expression
            let pos = expr.loc().begin_range();

            diagnostics.push(Diagnostic::error(
                pos,
                "code block missing for no catch".to_string(),
            ));
            return Err(());
        }
    };

    symtable.new_scope();

    let mut args = match &fcall {
        Expression::ExternalFunctionCall {
            returns: func_returns,
            ..
        } => {
            let mut func_returns = func_returns.clone();

            if func_returns == vec![Type::Void] {
                func_returns = vec![];
            }

            if returns.len() != func_returns.len() {
                diagnostics.push(Diagnostic::error(
                    expr.loc(),
                    format!(
                        "try returns list has {} entries while function returns {} values",
                        returns.len(),
                        func_returns.len()
                    ),
                ));
                return Err(());
            }

            func_returns
        }
        Expression::Constructor { contract_no, .. } => match returns.len() {
            0 => Vec::new(),
            1 => vec![Type::Contract(*contract_no)],
            _ => {
                diagnostics.push(Diagnostic::error(
                    expr.loc(),
                    format!(
                        "constructor returns single contract, not {} values",
                        returns.len()
                    ),
                ));
                return Err(());
            }
        },
        _ => {
            diagnostics.push(Diagnostic::error(
                expr.loc(),
                "try only supports external calls or constructor calls".to_string(),
            ));
            return Err(());
        }
    };

    symtable.new_scope();

    let mut params = Vec::new();
    let mut broken = false;
    for param in returns {
        let arg_ty = args.remove(0);

        match &param.1 {
            Some(program::Parameter {
                ty, storage, name, ..
            }) => {
                let (ret_ty, ty_loc) = resolve_var_decl_ty(ty, storage, context, ns, diagnostics)?;

                if arg_ty != ret_ty {
                    diagnostics.push(Diagnostic::error(
                        ty.loc(),
                        format!(
                            "type '{}' does not match return value of function '{}'",
                            ret_ty.to_string(ns),
                            arg_ty.to_string(ns)
                        ),
                    ));
                    broken = true;
                }

                if let Some(name) = name {
                    if let Some(pos) = symtable.add(
                        name,
                        ret_ty.clone(),
                        ns,
                        VariableInitializer::Solidity(None),
                        VariableUsage::TryCatchReturns,
                        storage.clone(),
                    ) {
                        ns.check_shadowing(context.file_no, context.contract_no, name);
                        params.push((
                            Some(pos),
                            Parameter {
                                loc: param.0,
                                ty: ret_ty,
                                ty_loc: Some(ty_loc),
                                id: Some(name.clone()),
                                recursive: false,
                            },
                        ));
                    }
                } else {
                    params.push((
                        None,
                        Parameter {
                            loc: param.0,
                            ty: ret_ty,
                            ty_loc: Some(ty_loc),
                            id: None,
                            recursive: false,
                        },
                    ));
                }
            }
            None => {
                diagnostics.push(Diagnostic::error(
                    param.0,
                    "missing return type".to_string(),
                ));
                broken = true;
            }
        }
    }

    if broken {
        return Err(());
    }

    let mut ok_resolved = Vec::new();

    let mut finally_reachable = statement(
        ok,
        &mut ok_resolved,
        context,
        symtable,
        loops,
        ns,
        diagnostics,
    )?;

    symtable.leave_scope();

    let mut clauses_unique = HashSet::new();
    let mut errors_resolved = Vec::new();
    let mut catch_param = None;
    let mut catch_param_pos = None;
    let mut catch_stmt_resolved = Vec::new();

    clause_stmts.iter().try_for_each(|clause_stmt| {
        let (loc, name) = match clause_stmt {
            CatchClause::Simple(loc, _, _) => (loc, ""),
            CatchClause::Named(loc, id, _, _) => (loc, id.name.as_str()),
        };
        if !clauses_unique.insert(name) {
            ns.diagnostics.push(Diagnostic::error(
                *loc,
                if name.is_empty() {
                    "duplicate catch clause".to_string()
                } else {
                    format!("duplicate '{}' catch clause", name)
                },
            ));
            return Err(());
        }

        match clause_stmt {
            CatchClause::Simple(_, param, stmt) => {
                symtable.new_scope();

                if let Some(param) = param {
                    let (catch_ty, ty_loc) =
                        resolve_var_decl_ty(&param.ty, &param.storage, context, ns, diagnostics)?;

                    if catch_ty != Type::DynamicBytes {
                        diagnostics.push(Diagnostic::error(
                            param.ty.loc(),
                            format!(
                                "catch can only take 'bytes memory', not '{}'",
                                catch_ty.to_string(ns)
                            ),
                        ));
                        return Err(());
                    }

                    let mut result = Parameter {
                        loc: param.loc,
                        ty: Type::DynamicBytes,
                        ty_loc: Some(ty_loc),
                        id: None,
                        recursive: false,
                    };

                    if let Some(name) = &param.name {
                        if let Some(pos) = symtable.add(
                            name,
                            catch_ty,
                            ns,
                            VariableInitializer::Solidity(None),
                            VariableUsage::TryCatchErrorBytes,
                            param.storage.clone(),
                        ) {
                            ns.check_shadowing(context.file_no, context.contract_no, name);
                            catch_param_pos = Some(pos);
                            result.id = Some(name.clone());
                        }
                    }

                    catch_param = Some(result);
                }

                let reachable = statement(
                    stmt,
                    &mut catch_stmt_resolved,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;

                finally_reachable |= reachable;

                symtable.leave_scope();

                Ok(())
            }
            CatchClause::Named(_, id, param, stmt) => {
                if !matches!(id.name.as_str(), "Error" | "Panic") {
                    ns.diagnostics.push(Diagnostic::error(
                        id.loc,
                        format!(
                            "only catch 'Error' or 'Panic' is supported, not '{}'",
                            id.name
                        ),
                    ));
                    return Err(());
                }

                let (error_ty, ty_loc) =
                    resolve_var_decl_ty(&param.ty, &param.storage, context, ns, diagnostics)?;

                if id.name == "Error" && error_ty != Type::String {
                    ns.diagnostics.push(Diagnostic::error(
                        param.ty.loc(),
                        format!(
                            "catch Error(...) can only take 'string memory', not '{}'",
                            error_ty.to_string(ns)
                        ),
                    ));
                }
                if id.name == "Panic" && error_ty != Type::Uint(256) {
                    ns.diagnostics.push(Diagnostic::error(
                        param.ty.loc(),
                        format!(
                            "catch Panic(...) can only take 'uint256', not '{}'",
                            error_ty.to_string(ns)
                        ),
                    ));
                }

                symtable.new_scope();

                let mut error_pos = None;
                let mut error_stmt_resolved = Vec::new();
                let mut error_param = Parameter {
                    loc: id.loc,
                    ty: Type::String,
                    ty_loc: Some(ty_loc),
                    id: None,
                    recursive: false,
                };

                if let Some(name) = &param.name {
                    if let Some(pos) = symtable.add(
                        name,
                        Type::String,
                        ns,
                        VariableInitializer::Solidity(None),
                        VariableUsage::TryCatchErrorString,
                        param.storage.clone(),
                    ) {
                        ns.check_shadowing(context.file_no, context.contract_no, name);

                        error_pos = Some(pos);
                        error_param.id = Some(name.clone());
                    }
                }

                let reachable = statement(
                    stmt,
                    &mut error_stmt_resolved,
                    context,
                    symtable,
                    loops,
                    ns,
                    diagnostics,
                )?;

                finally_reachable |= reachable;

                symtable.leave_scope();

                errors_resolved.push((error_pos, error_param, error_stmt_resolved));

                Ok(())
            }
        }
    })?;

    let stmt = Statement::TryCatch(
        *loc,
        finally_reachable,
        TryCatch {
            expr: fcall,
            returns: params,
            errors: errors_resolved,
            ok_stmt: ok_resolved,
            catch_param,
            catch_param_pos,
            catch_stmt: catch_stmt_resolved,
        },
    );

    Ok((stmt, finally_reachable))
}
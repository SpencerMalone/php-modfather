use bumpalo::Bump;
use mago_database::file::FileId;
use mago_syntax::ast::*;
use mago_syntax::parser::parse_file_content;

fn main() {
    let arena = Bump::new();

    let code = r#"<?php
namespace App;

use Other\Logger;

class Test {
    public function run() {
        $user = new User();
        Logger::info("test");
        if ($user instanceof User) {}
        try {} catch (Exception $e) {}
    }
}
"#;

    let result = parse_file_content(&arena, FileId::zero(), code.as_bytes());

    match result {
        Ok(program) => {
            println!("=== AST Structure ===\n");
            for statement in program.statements.iter() {
                explore_statement(statement, 0);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}

fn explore_statement(stmt: &Statement, indent: usize) {
    let prefix = "  ".repeat(indent);

    match stmt {
        Statement::Class(class) => {
            println!("{}Class: {}", prefix, class.name.value);
            for member in class.members.iter() {
                explore_member(member, indent + 1);
            }
        }
        Statement::Namespace(ns) => {
            println!("{}Namespace", prefix);
            for stmt in ns.statements().iter() {
                explore_statement(stmt, indent + 1);
            }
        }
        _ => {}
    }
}

fn explore_member(member: &ClassLikeMember, indent: usize) {
    let prefix = "  ".repeat(indent);

    match member {
        ClassLikeMember::Method(method) => {
            println!("{}Method: {}", prefix, method.name.value);
            println!("{}  Body present: {}", prefix, method.body.is_some());

            if let Some(body) = &method.body {
                println!("{}  Body type: FunctionLikeBody", prefix);
                explore_body(body, indent + 1);
            }
        }
        _ => {}
    }
}

fn explore_body(body: &FunctionLikeBody, indent: usize) {
    let prefix = "  ".repeat(indent);

    match body {
        FunctionLikeBody::Block(block) => {
            println!("{}Block with {} statements", prefix, block.statements.len());
            for stmt in block.statements.iter() {
                explore_body_statement(stmt, indent + 1);
            }
        }
        _ => {}
    }
}

fn explore_body_statement(stmt: &Statement, indent: usize) {
    let prefix = "  ".repeat(indent);

    match stmt {
        Statement::Expression(expr_stmt) => {
            println!("{}ExpressionStatement", prefix);
            explore_expression(&expr_stmt.expression, indent + 1);
        }
        Statement::If(if_stmt) => {
            println!("{}If", prefix);
            explore_expression(&if_stmt.condition, indent + 1);
        }
        Statement::Try(try_stmt) => {
            println!("{}Try", prefix);
            for clause in try_stmt.catch_clauses.iter() {
                explore_catch_clause(clause, indent + 1);
            }
        }
        _ => {
            println!("{}{:?}", prefix, std::mem::discriminant(stmt));
        }
    }
}

fn explore_expression(expr: &Expression, indent: usize) {
    let prefix = "  ".repeat(indent);

    match expr {
        Expression::Instantiation(inst) => {
            println!("{}Instantiation", prefix);
            println!("{}  Class: {:?}", prefix, inst.class);
        }
        Expression::StaticMethodCall(call) => {
            println!("{}StaticMethodCall", prefix);
            println!("{}  Class: {:?}", prefix, call.class);
            println!("{}  Method: {:?}", prefix, call.method);
        }
        Expression::StaticPropertyFetch(fetch) => {
            println!("{}StaticPropertyFetch", prefix);
            println!("{}  Class: {:?}", prefix, fetch.class);
        }
        Expression::Instanceof(inst) => {
            println!("{}Instanceof", prefix);
            explore_expression(&inst.left, indent + 1);
            explore_expression(&inst.right, indent + 1);
        }
        Expression::Identifier(id) => {
            println!("{}Identifier: {}", prefix, id.value());
        }
        Expression::Variable(var) => {
            println!("{}Variable: {:?}", prefix, var.name);
        }
        Expression::Assignment(assign) => {
            println!("{}Assignment", prefix);
            explore_expression(&assign.right, indent + 1);
        }
        _ => {
            println!("{}{:?}", prefix, std::mem::discriminant(expr));
        }
    }
}

fn explore_catch_clause(clause: &CatchClause, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}CatchClause", prefix);
    println!("{}  Types: {:?}", prefix, clause.types);
}

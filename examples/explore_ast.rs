use bumpalo::Bump;
use mago_database::file::FileId;
use mago_syntax::ast::*;
use mago_syntax::parser::parse_file_content;

fn main() {
    let arena = Bump::new();

    let php_code = r#"<?php
namespace App\Controllers;

use App\Models\User;
use App\Services\UserService as Service;

class UserController {
    private Service $service;

    public function getUser(int $id): User {
        return $this->service->findUser($id);
    }
}
"#;

    let file_id = FileId::zero();
    let (program, error) = parse_file_content(&arena, file_id, php_code);

    if let Some(err) = error {
        eprintln!("Parse error: {:?}", err);
    }

    println!("Program has {} statements", program.statements.len());

    for (i, statement) in program.statements.iter().enumerate() {
        println!("\nStatement {}: {:?}", i, std::mem::discriminant(statement));

        match statement {
            Statement::Namespace(ns) => {
                println!("  Namespace found");
                println!("    Name: {:?}", ns.name);
                println!("    Has {} statements", ns.statements().len());

                for (j, stmt) in ns.statements().iter().enumerate() {
                    println!("    Statement {}: {:?}", j, std::mem::discriminant(stmt));

                    if let Statement::Use(use_stmt) = stmt {
                        println!("      USE STATEMENT: {:?}", use_stmt);
                    }

                    if let Statement::Class(class) = stmt {
                        println!("      Class name: {:?}", class.name);
                        println!("      Has extends: {}", class.extends.is_some());
                        println!("      Has implements: {}", class.implements.is_some());

                        if let Some(extends) = &class.extends {
                            println!("      Extends: {:?}", extends);
                        }

                        if let Some(implements) = &class.implements {
                            println!("      Implements: {} interfaces", implements.types.len());
                        }

                        println!("      Has {} members", class.members.len());
                        for (k, member) in class.members.iter().enumerate() {
                            println!("        Member {}: {:?}", k, std::mem::discriminant(member));
                            match member {
                                ClassLikeMember::Property(prop) => {
                                    println!("          Property: {:?}", prop);
                                }
                                ClassLikeMember::Method(method) => {
                                    println!("          Method params: {} parameters", method.parameter_list.parameters.len());
                                    if !method.parameter_list.parameters.is_empty() {
                                        println!("          First param: {:?}", method.parameter_list.parameters.first());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

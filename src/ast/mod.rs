
use std::hash::Hash;
use std::string;


use termion::color::{Fg, Reset};

use fusion_compiler::{Idx, idx, IdxVec};
use printer::ASTPrinter;
use visitor::ASTVisitor;

use crate::ast::lexer::{ Token, StringFragment, StringKind };

use crate::compilation_unit::{FunctionIdx, VariableIdx};
use crate::text::span::TextSpan;
use crate::typings::Type;

pub mod lexer;
pub mod parser;
pub mod evaluator;
pub mod visitor;
pub mod printer;

idx!(StmtId);
idx!(ExprId);
idx!(ItemId);

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
pub struct Ast {
    pub statements: IdxVec<StmtId, Stmt>,
    pub expressions: IdxVec<ExprId, Expr>,
    pub items: IdxVec<ItemId, Item>,
}

impl Ast {
    pub fn new() -> Self {
        Self { statements: IdxVec::new(), expressions: IdxVec::new(), items: IdxVec::new() }
    }

    pub fn query_item(&self, item_id: ItemId) -> &Item {
        &self.items[item_id]
    }

    pub fn query_expr(&self, expr_id: ExprId) -> &Expr {
        &self.expressions[expr_id]
    }

    fn query_expr_mut(&mut self, expr_id: ExprId) -> &mut Expr {
        &mut self.expressions[expr_id]
    }

    pub fn query_stmt(&self, stmt_id: StmtId) -> &Stmt {
        &self.statements[stmt_id]
    }

    fn query_stmt_mut(&mut self, stmt_id: StmtId) -> &mut Stmt {
        &mut self.statements[stmt_id]
    }
    
    pub fn set_variable(&mut self, expr_id: ExprId, variable_idx: VariableIdx) {
        let expr = self.query_expr_mut(expr_id);
        match &mut expr.kind {
            ExprKind::Assignment(assign_expr) => {
                assign_expr.variable_idx = variable_idx;
            }
            ExprKind::Variable(var_expr) => {
                var_expr.variable_idx = variable_idx;
            }
            _ => unreachable!("Cannot set variable of non-variable expression")
        }
    }

    pub fn set_variable_for_stmt(&mut self, stmt_id: &StmtId, variable_idx: VariableIdx) {
        let stmt = self.query_stmt_mut(*stmt_id);
        match &mut stmt.kind {
            StmtKind::Let(var_decl) => {
                var_decl.variable_idx = variable_idx;
            }
            _ => unreachable!("Cannot set variable of non-variable statement")
        }
    }

    pub fn set_type(&mut self, expr_id: ExprId, ty: Type) {
        let expr = &mut self.expressions[expr_id];
        expr.ty = ty;
    }

    fn stmt_from_kind(&mut self, kind: StmtKind) -> &Stmt {
        let stmt = Stmt::new(kind, StmtId::new(0));
        let id = self.statements.push(stmt);
        self.statements[id].id = id;
        &self.statements[id]
    }

    pub fn expression_statement(&mut self, expr_id: ExprId) -> &Stmt {
        self.stmt_from_kind(StmtKind::Expr(expr_id))
    }

    pub fn let_statement(&mut self, identifier: Token, initializer: ExprId, type_annotation: Option<StaticTypeAnnotation>) -> &Stmt {
        self.stmt_from_kind(StmtKind::Let(LetStmt { identifier, initializer, type_annotation, variable_idx: VariableIdx::new(0) }))
    }

    pub fn if_expr(&mut self, if_keyword: Token, condition: ExprId, then: ExprId, else_statement: Option<ElseBranch>) -> &Expr {
        self.expr_from_kind(ExprKind::If(IfExpr { if_keyword, condition, then_branch: then, else_branch: else_statement }))
    }

    pub fn while_statement(&mut self, while_keyword: Token, condition: ExprId, body: ExprId) -> &Stmt {
        self.stmt_from_kind(StmtKind::While(WhileStmt { while_keyword, condition, body }))
    }

    pub fn block_expression(&mut self, left_brace: Token, statements: Vec<StmtId>, right_brace: Token) -> &Expr {
        self.expr_from_kind(ExprKind::Block(BlockExpr { left_brace, stmts: statements, right_brace }))
    }

    pub fn return_statement(&mut self, return_keyword: Token, return_value: Option<ExprId>) -> &Stmt {
        self.stmt_from_kind(StmtKind::Return(ReturnStmt { return_keyword, return_value }))
    }

    pub fn func_item(&mut self, func_keyword: Token, identifier: Token, parameters: Vec<FuncDeclParameter>, body: ExprId, return_type: Option<FunctionReturnTypeSyntax>, function_idx: FunctionIdx) -> &Item {
        return self.item_from_kind(
            ItemKind::Function(
                FunctionDeclaration {
                    func_keyword,
                    identifier,
                    parameters,
                    body,
                    return_type,
                    idx: function_idx,
                }
            )
        );
    }

    pub fn item_from_kind(&mut self, kind: ItemKind) -> &Item {
        let item = Item::new(kind, ItemId::new(0));
        let id = self.items.push(item);
        self.items[id].id = id;
        &self.items[id]
    }

    fn expr_from_kind(&mut self, kind: ExprKind) -> &Expr {
        let expr = Expr::new(kind, ExprId::new(0), Type::Unresolved);
        let id = self.expressions.push(expr);
        self.expressions[id].id = id;
        &self.expressions[id]
    }

    pub fn number_expression(&mut self, token: Token, number: i64) -> &Expr {
        self.expr_from_kind(ExprKind::Number(NumberExpr { number, token }))
    }
    
    pub fn decimal_expression(&mut self, token:Token, number: f64) -> &Expr {
        self.expr_from_kind(ExprKind::Decimal(DecimalExpr { number, token }))
    }

    pub fn string_expression(&mut self, token:Token, string: String) -> &Expr {
        self.expr_from_kind(ExprKind::String(StringExpr { string, token }))
    }

    pub fn binary_expression(&mut self, operator: BinOperator, left: ExprId, right: ExprId) -> &Expr {
        self.expr_from_kind(ExprKind::Binary(BinaryExpr { operator, left, right }))
    }

    pub fn parenthesized_expression(&mut self, left_paren: Token, expression: ExprId, right_paren: Token) -> &Expr {
        self.expr_from_kind(ExprKind::Parenthesized(ParenthesizedExpr { expression, left_paren, right_paren }))
    }

    pub fn variable_expression(&mut self, identifier: Token) -> &Expr {
        self.expr_from_kind(ExprKind::Variable(VarExpr { identifier, variable_idx: VariableIdx::new(0) }))
    }

    pub fn unary_expression(&mut self, operator: UnOperator, operand: ExprId) -> &Expr {
        self.expr_from_kind(ExprKind::Unary(UnaryExpr { operator, operand }))
    }

    pub fn assignment_expression(&mut self, identifier: Token, equals: Token, expression: ExprId) -> &Expr {
        self.expr_from_kind(ExprKind::Assignment(AssignExpr { identifier, expression, equals, variable_idx: VariableIdx::new(0) }))
    }

    pub fn boolean_expression(&mut self, token: Token, value: bool) -> &Expr {
        self.expr_from_kind(ExprKind::Boolean(BoolExpr { token, value }))
    }

    pub fn call_expression(&mut self, callee: Token, left_paren: Token, arguments: Vec<ExprId>, right_paren: Token) -> &Expr {
        self.expr_from_kind(ExprKind::Call(CallExpr { callee, arguments, left_paren, right_paren }))
    }

    pub fn error_expression(&mut self, span: TextSpan) -> &Expr {
        self.expr_from_kind(ExprKind::Error(span))
    }

    pub fn visit(&mut self, visitor: &mut dyn ASTVisitor) {
        for item in self.items.clone().iter() {
            visitor.visit_item(self, item.id);
        }
    }

    pub fn visualize(&mut self) -> () {
        let mut printer = ASTPrinter::new();
        self.visit(&mut printer);
        println!("{}", printer.result);
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
}

impl Item {
    pub fn new(kind: ItemKind, id: ItemId) -> Self {
        Self { kind, id }
    }
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Stmt(StmtId),
    Function(FunctionDeclaration),
}


#[derive(Debug, Clone)]
pub enum StmtKind {
    Expr(ExprId),
    Let(LetStmt),
    While(WhileStmt),
    Return(ReturnStmt),
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub return_keyword: Token,
    pub return_value: Option<ExprId>,
}

#[derive(Debug, Clone)]
pub struct StaticTypeAnnotation {
    pub colon: Token,
    pub type_name: Token,
}

impl StaticTypeAnnotation {
    pub fn new(colon: Token, type_name: Token) -> Self {
        Self { colon, type_name }
    }
}

#[derive(Debug, Clone)]
pub struct FuncDeclParameter {
    pub identifier: Token,
    pub type_annotation: StaticTypeAnnotation,
}

#[derive(Debug, Clone)]
pub struct FunctionReturnTypeSyntax {
    pub arrow: Token,
    pub type_name: Token,
}

impl FunctionReturnTypeSyntax {
    pub fn new(arrow: Token, type_name: Token) -> Self {
        Self { arrow, type_name }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub func_keyword: Token,
    pub identifier: Token,
    pub parameters: Vec<FuncDeclParameter>,
    pub body: ExprId,
    pub return_type: Option<FunctionReturnTypeSyntax>,
    pub idx: FunctionIdx,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub while_keyword: Token,
    pub condition: ExprId,
    pub body: ExprId,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub left_brace: Token,
    pub stmts: Vec<StmtId>,
    pub right_brace: Token,
}

#[derive(Debug, Clone)]
pub struct ElseBranch {
    pub else_keyword: Token,
    pub expr: ExprId,
}

impl ElseBranch {
    pub fn new(else_keyword: Token, expr: ExprId) -> Self {
        ElseBranch { else_keyword, expr }
    }
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub if_keyword: Token,
    pub condition: ExprId,
    pub then_branch: ExprId,
    pub else_branch: Option<ElseBranch>,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub identifier: Token,
    pub initializer: ExprId,
    pub type_annotation: Option<StaticTypeAnnotation>,
    pub variable_idx: VariableIdx,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub id: StmtId,
}

impl Stmt {
    pub fn new(kind: StmtKind, id: StmtId) -> Self {
        Stmt { kind, id }
    }

    pub fn span(&self, ast: &Ast) -> TextSpan {
        match &self.kind {
            StmtKind::Expr(expr_id) => ast.query_expr(*expr_id).span(ast),
            StmtKind::Let(let_stmt) => {
                let mut spans = vec![let_stmt.identifier.span.clone()];
                if let Some(type_annotation) = &let_stmt.type_annotation {
                    spans.push(type_annotation.colon.span.clone());
                    spans.push(type_annotation.type_name.span.clone());
                }
                TextSpan::combine(spans)
            }
            StmtKind::While(while_stmt) => {
                let mut spans = vec![while_stmt.while_keyword.span.clone()];
                spans.push(ast.query_expr(while_stmt.condition).span(ast));
                spans.push(ast.query_expr(while_stmt.body).span(ast));
                TextSpan::combine(spans)
            }
            StmtKind::Return(return_stmt) => {
                let mut spans = vec![return_stmt.return_keyword.span.clone()];
                if let Some(return_value) = &return_stmt.return_value {
                    spans.push(ast.query_expr(*return_value).span(ast));
                }
                TextSpan::combine(spans)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Number(
        NumberExpr
    ),
    Decimal(
        DecimalExpr
    ),
    String(
        StringExpr
    ),
    Binary(
        BinaryExpr
    ),
    Unary(
        UnaryExpr
    ),
    Parenthesized(
        ParenthesizedExpr
    ),
    Variable(
        VarExpr
    ),
    Assignment(
        AssignExpr
    ),
    Boolean(
        BoolExpr
    ),
    Call(
        CallExpr
    ),
    If(IfExpr),
    Block(BlockExpr),
    Error(
        TextSpan
    ),
}


#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Token,
    pub left_paren: Token,
    pub arguments: Vec<ExprId>,
    pub right_paren: Token,
}

impl CallExpr {

    pub fn function_name(&self) -> &str {
        &self.callee.span.literal
    }

}

#[derive(Debug, Clone)]
pub struct BoolExpr {
    pub value: bool,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub identifier: Token,
    pub equals: Token,
    pub expression: ExprId,
    pub variable_idx: VariableIdx,

}

#[derive(Debug, Clone)]
pub enum UnOpKind {
    Minus,
    BitwiseNot,
}

#[derive(Debug, Clone)]
pub struct UnOperator {
    pub(crate) kind: UnOpKind,
    token: Token,
}

impl UnOperator {
    pub fn new(kind: UnOpKind, token: Token) -> Self {
        UnOperator { kind, token }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: UnOperator,
    pub operand: ExprId,
}

#[derive(Debug, Clone)]
pub struct VarExpr {
    pub identifier: Token,
    pub variable_idx: VariableIdx,
}

impl VarExpr {
    pub fn identifier(&self) -> &str {
        &self.identifier.span.literal
    }
}


#[derive(Debug, Clone)]
pub enum BinOpKind {
    // Arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    //float arithmetic
    PlusDecimal,
    MinusDecimal,
    MultiplyDecimal,
    DivideDecimal,
    //string arithmetic
    PlusString,
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    // Relational
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpAssociativity {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct BinOperator {
    pub kind: BinOpKind,
    pub token: Token,
}

impl BinOperator {
    pub fn new(kind: BinOpKind, token: Token) -> Self {
        BinOperator { kind, token }
    }

    pub fn precedence(&self) -> u8 {
        match self.kind {
            BinOpKind::Power => 20,
            BinOpKind::Multiply => 19,
            BinOpKind::Divide => 19,
            BinOpKind::Plus => 18,
            BinOpKind::PlusDecimal => 18,
            BinOpKind::MinusDecimal => 18,
            BinOpKind::MultiplyDecimal => 18,
            BinOpKind::DivideDecimal => 18,
            BinOpKind::PlusString => 18,
            BinOpKind::Minus => 18,
            BinOpKind::BitwiseAnd => 17,
            BinOpKind::BitwiseXor => 16,
            BinOpKind::BitwiseOr => 15,
            BinOpKind::Equals => 30,
            BinOpKind::NotEquals => 30,
            BinOpKind::LessThan => 29,
            BinOpKind::LessThanOrEqual => 29,
            BinOpKind::GreaterThan => 29,
            BinOpKind::GreaterThanOrEqual => 29,
        }
    }

    pub fn associativity(&self) -> BinOpAssociativity {
        match self.kind {
            BinOpKind::Power => BinOpAssociativity::Right,
            _ => BinOpAssociativity::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: ExprId,
    pub operator: BinOperator,
    pub right: ExprId,
}

#[derive(Debug, Clone)]
pub struct NumberExpr {
    pub number: i64,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct DecimalExpr{
    pub number: f64,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct StringExpr {
    pub string: String,
    pub token: Token,
}

#[derive(Debug, Clone)]
pub struct ParenthesizedExpr {
    pub left_paren: Token,
    pub expression: ExprId,
    pub right_paren: Token,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub id: ExprId,
    pub ty: Type,
}

impl Expr {
    pub fn new(kind: ExprKind, id: ExprId, ty: Type) -> Self {
        Expr { kind, id, ty }
    }

    pub fn span(&self, ast: &Ast) -> TextSpan {
        match &self.kind {
            ExprKind::Block(block_stmt) => {
                let mut spans = vec![block_stmt.left_brace.span.clone()];
                for statement in &block_stmt.stmts {
                    spans.push(ast.query_stmt(*statement).span(ast));
                }
                spans.push(block_stmt.right_brace.span.clone());
                TextSpan::combine(spans)
            }

            ExprKind::Number(expr) => expr.token.span.clone(),
            ExprKind::Binary(expr) => {
                let left = ast.query_expr(expr.left).span(ast);
                let operator = expr.operator.token.span.clone();
                let right = ast.query_expr(expr.right).span(ast);
                TextSpan::combine(vec![left, operator, right])
            }
            ExprKind::Decimal(expr) => expr.token.span.clone(),
            ExprKind::String(expr) => expr.token.span.clone(),
            ExprKind::Unary(expr) => {
                let operator = expr.operator.token.span.clone();
                let operand = ast.query_expr(expr.operand).span(ast);
                TextSpan::combine(vec![operator, operand])
            }
            ExprKind::Parenthesized(expr) => {
                let open_paren = expr.left_paren.span.clone();
                let expression = ast.query_expr(expr.expression).span(ast);
                let close_paren = expr.right_paren.span.clone();
                TextSpan::combine(vec![open_paren, expression, close_paren])
            }
            ExprKind::Variable(expr) => expr.identifier.span.clone(),
            ExprKind::Assignment(expr) => {
                let identifier = expr.identifier.span.clone();
                let equals = expr.equals.span.clone();
                let expression = ast.query_expr(expr.expression).span(ast);
                TextSpan::combine(vec![identifier, equals, expression])
            }
            ExprKind::Boolean(expr) => expr.token.span.clone(),
            ExprKind::Call(expr) => {
                let callee_span = expr.callee.span.clone();
                let left_paren = expr.left_paren.span.clone();
                let right_paren = expr.right_paren.span.clone();
                let mut spans = vec![callee_span, left_paren, right_paren];
                for arg in &expr.arguments {
                    spans.push(ast.query_expr(*arg).span(ast));
                }
                TextSpan::combine(spans)
            }
            ExprKind::If(expr) => {
                let if_span = expr.if_keyword.span.clone();
                let condition = ast.query_expr(expr.condition).span(ast);
                let then_branch = ast.query_expr(expr.then_branch).span(ast);
                let mut spans = vec![if_span, condition, then_branch];
                if let Some(else_branch) = &expr.else_branch {
                    let else_span = else_branch.else_keyword.span.clone();
                    spans.push(else_span);
                    spans.push(ast.query_expr(else_branch.expr).span(ast));
                }
                TextSpan::combine(spans)
            }
            ExprKind::Error(span) => span.clone(),

           
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{AssignExpr, Ast, BinaryExpr, BlockExpr, BoolExpr, CallExpr, Expr, FunctionDeclaration, IfExpr, LetStmt, NumberExpr,StringExpr,DecimalExpr, ParenthesizedExpr, ReturnStmt, Stmt, UnaryExpr, VarExpr, WhileStmt};
    use crate::compilation_unit::CompilationUnit;
    use crate::text::span::TextSpan;

    use super::visitor::ASTVisitor;

    #[derive(Debug)]
    enum TestASTNode {
        Number(i64),
        Decimal(f64),
        String(String),
        Boolean(bool),
        Binary,
        Unary,
        Parenthesized,
        Let,
        Assignment,
        Block,
        Variable(String),
        If,
        Else,
        Func,
        While,
        Return,
        Call,
    }

    struct ASTVerifier {
        expected: Vec<TestASTNode>,
        actual: Vec<TestASTNode>,
        ast: Ast,
    }

    impl Eq for TestASTNode {}

    impl PartialEq for TestASTNode{
        fn eq(&self,other:&Self) -> bool{
            match (self, other) {
                (TestASTNode::Number(n1), TestASTNode::Number(n2)) => n1 == n2,
                (TestASTNode::Decimal(n1), TestASTNode::Decimal(n2)) => {
                    let diff = (n1 - n2).abs();
                    diff < 0.0000000000001 //floating point precision 
                },
                (TestASTNode::String(s1), TestASTNode::String(s2)) => s1 == s2,
                (TestASTNode::Boolean(b1), TestASTNode::Boolean(b2)) => b1 == b2,
                _ => false,
            }
        }
    }
    

    impl ASTVerifier {
        pub fn new(input: &str, expected: Vec<TestASTNode>) -> Self {
            let compilation_unit = CompilationUnit::compile(input).expect("Failed to compile");
            let mut verifier = ASTVerifier { expected, actual: Vec::new(), ast: compilation_unit.ast };
            verifier.flatten_ast();
            verifier
        }

        fn flatten_ast(&mut self) {
            self.actual.clear();
            let mut ast = self.ast.clone();
            ast.visit(&mut *self);
        }

        pub fn verify(&self) {
            assert_eq!(self.expected.len(), self.actual.len(), "Expected {} nodes, but got {}. Actual nodes: {:?}", self.expected.len(), self.actual.len(), self.actual);

            for (index, (expected, actual)) in self.expected.iter().zip(
                self.actual.iter()
            ).enumerate() {
                assert_eq!(expected, actual, "Expected {:?} at index {}, but got {:?}", expected, index, actual);
            }
        }
    }

    impl ASTVisitor for ASTVerifier {
        fn visit_return_statement(&mut self, ast: &mut Ast, return_statement: &ReturnStmt) {
            self.actual.push(TestASTNode::Return);
            if let Some(expression) = &return_statement.return_value {
                self.visit_expression(ast, *expression);
            }
        }

        fn visit_while_statement(&mut self, ast: &mut Ast, while_statement: &WhileStmt) {
            self.actual.push(TestASTNode::While);
            self.visit_expression(ast, while_statement.condition);
            self.visit_expression(ast, while_statement.body);
        }

        fn visit_block_expr(&mut self, ast: &mut Ast, block_statement: &BlockExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Block);
            for statement in &block_statement.stmts {
                self.visit_statement(ast, *statement);
            }
        }

        fn visit_if_expression(&mut self, ast: &mut Ast, if_statement: &IfExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::If);
            self.visit_expression(ast, if_statement.condition);
            self.visit_expression(ast, if_statement.then_branch);
            if let Some(else_branch) = &if_statement.else_branch {
                self.actual.push(TestASTNode::Else);

                self.visit_expression(ast, else_branch.expr);
            }
        }

        fn visit_let_statement(&mut self, ast: &mut Ast, let_statement: &LetStmt, _stmt: &Stmt) {
            self.actual.push(TestASTNode::Let);
            self.visit_expression(ast, let_statement.initializer);
        }

        fn visit_call_expression(&mut self, ast: &mut Ast, call_expression: &CallExpr,_expr: &Expr) {
            self.actual.push(TestASTNode::Call);
            for argument in &call_expression.arguments {
                self.visit_expression(ast, *argument);
            }
        }

        fn visit_assignment_expression(&mut self, ast: &mut Ast, assignment_expression: &AssignExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Assignment);
            self.visit_expression(ast, assignment_expression.expression);
        }

        fn visit_variable_expression(&mut self, _ast: &mut Ast, variable_expression: &VarExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Variable(
                variable_expression.identifier().to_string()
            ));
        }

        fn visit_number_expression(&mut self, _ast: &mut Ast, number: &NumberExpr,_expr: &Expr) {
            self.actual.push(TestASTNode::Number(number.number));
        }

        fn visit_decimal_expression(&mut self, _ast:&mut Ast, decimal: &DecimalExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Decimal(decimal.number));
        }

        fn visit_string_expression(&mut self, ast: &mut Ast, string: &StringExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::String(string.string.clone()));
        }

        fn visit_boolean_expression(&mut self, _ast: &mut Ast, boolean: &BoolExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Boolean(boolean.value));
        }

        fn visit_error(&mut self, _ast: &mut Ast, _span: &TextSpan) {
            // do nothing
        }

        fn visit_unary_expression(&mut self, ast: &mut Ast, unary_expression: &UnaryExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Unary);
            self.visit_expression(ast, unary_expression.operand);
        }

        fn visit_binary_expression(&mut self, ast: &mut Ast, binary_expression: &BinaryExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Binary);
            self.visit_expression(ast, binary_expression.left);
            self.visit_expression(ast, binary_expression.right);
        }

        fn visit_parenthesized_expression(&mut self, ast: &mut Ast, parenthesized_expression: &ParenthesizedExpr, _expr: &Expr) {
            self.actual.push(TestASTNode::Parenthesized);
            self.visit_expression(ast, parenthesized_expression.expression);
        }

        fn visit_func_decl(&mut self, _ast: &mut Ast, _func_decl: &FunctionDeclaration, _item_id: super::ItemId) {
            todo!()
        }

        fn visit_item(&mut self, ast: &mut Ast, item: super::ItemId) {
            self.visit_item_default(ast, item);
        }

        fn visit_item_default(&mut self, ast: &mut Ast, item: super::ItemId) {
            let item = ast.query_item(item).clone();
            match &item.kind {
                super::ItemKind::Stmt(stmt) => {
                    self.visit_statement(ast, *stmt);
                }
                super::ItemKind::Function(func_decl) => {
                    self.visit_func_decl(ast, func_decl, item.id);
                }
            }
        }

        fn do_visit_statement(&mut self, ast: &mut Ast, statement: super::StmtId) {
            let statement = ast.query_stmt(statement).clone();
            match &statement.kind {
                super::StmtKind::Expr(expr) => {
                    self.visit_expression(ast, *expr);
                }
                super::StmtKind::Let(expr) => {
                    self.visit_let_statement(ast, expr, &statement);
                }
                super::StmtKind::While(stmt) => {
                    self.visit_while_statement(ast, &stmt);
                }
                super::StmtKind::Return(stmt) => {
                    self.visit_return_statement(ast, &stmt);
                }
            }
        }

        fn visit_statement(&mut self, ast: &mut Ast, statement: super::StmtId) {
            self.do_visit_statement(ast, statement);
        }

        fn do_visit_expression(&mut self, ast: &mut Ast, expression: super::ExprId) {
            let expression = ast.query_expr(expression).clone();
            match &expression.kind {
                super::ExprKind::Number(number) => {
                    self.visit_number_expression(ast, number, &expression);
                }
                super::ExprKind::Decimal(decimal) => {
                    self.visit_decimal_expression(ast, decimal, &expression);
                }
                super::ExprKind::String(string) => {
                    self.visit_string_expression(ast, string, &expression);
                }
                super::ExprKind::Binary(expr) => {
                    self.visit_binary_expression(ast, expr, &expression);
                }
                super::ExprKind::Parenthesized(expr) => {
                    self.visit_parenthesized_expression(ast, expr, &expression);
                }
                super::ExprKind::Error(span) => {
                    self.visit_error(ast, span);
                }
                super::ExprKind::Variable(expr) => {
                    self.visit_variable_expression(ast, expr, &expression);
                }
                super::ExprKind::Unary(expr) => {
                    self.visit_unary_expression(ast, expr, &expression);
                }
                super::ExprKind::Assignment(expr) => {
                    self.visit_assignment_expression(ast, expr, &expression);
                }
                super::ExprKind::Boolean(expr) => {
                    self.visit_boolean_expression(ast, expr, &expression);
                }
                super::ExprKind::Call(expr) => {
                    self.visit_call_expression(ast, expr, &expression);
                }
                super::ExprKind::If(expr) => {
                    self.visit_if_expression(ast, expr, &expression);
                }
                super::ExprKind::Block(block_expr) => {
                    self.visit_block_expr(ast, &block_expr, &expression);
                }
            }
        }

        fn visit_expression(&mut self, ast: &mut Ast, expression: super::ExprId) {
            self.do_visit_expression(ast, expression);
        }
    }


    fn assert_tree(input: &str, expected: Vec<TestASTNode>) {
        let verifier = ASTVerifier::new(input, expected);
        verifier.verify();
    }

    #[test]
    pub fn should_parse_basic_binary_expression() {
        let input = "let a = 1 + 2";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_parenthesized_binary_expression() {
        let input = "let a = (1 + 2) * 3";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Parenthesized,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
            TestASTNode::Number(3),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_parenthesized_binary_expression_with_variable() {
        let input = "\
        let b = 1
        let a = (1 + 2) * b";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Number(1),
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Parenthesized,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
            TestASTNode::Variable("b".to_string()),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_parenthesized_binary_expression_with_variable_and_number() {
        let input = "\
        let b = 1
        let a = (1 + 2) * b + 3";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Number(1),
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Binary,
            TestASTNode::Parenthesized,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
            TestASTNode::Variable("b".to_string()),
            TestASTNode::Number(3),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_bitwise_and() {
        let input = "let a = 1 & 2";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_bitwise_or() {
        let input = "let a = 1 | 2";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_bitwise_xor() {
        let input = "let a = 1 ^ 2";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_bitwise_not() {
        let input = "let a = ~1";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Unary,
            TestASTNode::Number(1),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_negation() {
        let input = "let a = -1";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Unary,
            TestASTNode::Number(1),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_power() {
        let input = "let a = 1 ** 2";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Number(1),
            TestASTNode::Number(2),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_hilarious_amount_of_unary_operators() {
        let input = "let a = -1 + -2 * -3 ** ------4";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Binary,
            TestASTNode::Unary,
            TestASTNode::Number(1),
            TestASTNode::Binary,
            TestASTNode::Unary,
            TestASTNode::Number(2),
            TestASTNode::Binary,
            TestASTNode::Unary,
            TestASTNode::Number(3),
            TestASTNode::Unary,
            TestASTNode::Unary,
            TestASTNode::Unary,
            TestASTNode::Unary,
            TestASTNode::Unary,
            TestASTNode::Unary,
            TestASTNode::Number(4),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_if_statement() {
        let input = "\
        let a = 1
        if a > 0 {
            a = 20
        }
        ";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Number(1),
            TestASTNode::If,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Number(0),
            TestASTNode::Block,
            TestASTNode::Assignment,
            TestASTNode::Number(20),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_if_statement_with_else() {
        let input = "\
        let a = 1
        if a > 0 {
            a = 20
        } else {
            a = 30
        }
        ";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Number(1),
            TestASTNode::If,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Number(0),
            TestASTNode::Block,
            TestASTNode::Assignment,
            TestASTNode::Number(20),
            TestASTNode::Else,
            TestASTNode::Block,
            TestASTNode::Assignment,
            TestASTNode::Number(30),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_while_statement() {
        let input = "\
        let a = 1
        while a < 10 {
            a = a + 1
        }
        ";
        let expected = vec![
            TestASTNode::Let,
            TestASTNode::Number(1),
            TestASTNode::While,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Number(10),
            TestASTNode::Block,
            TestASTNode::Assignment,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Number(1),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_function_declaration() {
        let input = "\
        func add(a: int, b: int) -> int {
            return a + b
        }
        ";
        let expected = vec![
            TestASTNode::Func,
            TestASTNode::Block,
            TestASTNode::Return,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Variable("b".to_string()),
        ];

        assert_tree(input, expected);
    }

    #[test]
    pub fn should_parse_call_expression() {
        let input = "\
        func add(a: int, b: int) -> int {
            return a + b
        }
        add(2 * 3, 4 + 5)";
        let expected = vec![
            TestASTNode::Func,
            TestASTNode::Block,
            TestASTNode::Return,
            TestASTNode::Binary,
            TestASTNode::Variable("a".to_string()),
            TestASTNode::Variable("b".to_string()),
            TestASTNode::Call,
            TestASTNode::Binary,
            TestASTNode::Number(2),
            TestASTNode::Number(3),
            TestASTNode::Binary,
            TestASTNode::Number(4),
            TestASTNode::Number(5),
        ];

        assert_tree(input, expected);
    }
}

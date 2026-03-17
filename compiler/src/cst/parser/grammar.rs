use crate::cst::parser::event::OpenMarker;
use crate::cst::parser::Parser;
use crate::cst::token::{TokenKind};
use crate::cst::tree::TreeKind;

impl<'text> Parser<'text> {
    pub fn file(&mut self) {
        let marker = self.open();
        if self.at(TokenKind::Module) {
            self.module();
        } else {
            self.error("expected 'module'");
        }
        self.items(None);
        self.close(marker, TreeKind::File);
    }

    pub fn module(&mut self) {
        assert!(self.at(TokenKind::Module));
        let marker = self.open();
        self.expect(TokenKind::Module);
        self.expect(TokenKind::Identifier);
        if self.at(TokenKind::LeftParentheses) {
            self.parameters();
        }
        self.expect(TokenKind::Semicolon);
        self.close(marker, TreeKind::Module);
    }

    const ITEM_FIRST: &'static [TokenKind] = &[TokenKind::Class, TokenKind::Constant, TokenKind::Function, TokenKind::Field];

    fn items(&mut self, until: Option<TokenKind>) {
        let marker = self.open();
        while !self.end_of_file() {
            if until.is_some_and(|until| self.at(until)) {
                break;
            }
            if self.at_any(Self::ITEM_FIRST).is_some() {
                self.item();
            } else {
                self.consume_with_error("expected 'item'");
            }
        }
        self.close(marker, TreeKind::Items);
    }

    fn item(&mut self) {
        assert!(self.at_any(Self::ITEM_FIRST).is_some());
        let marker = self.open();
        match self.at_any(Self::ITEM_FIRST) {
            Some(TokenKind::Class) => self.class(marker),
            Some(TokenKind::Function | TokenKind::Constant) => self.function(marker),
            Some(TokenKind::Field) => self.field(marker),
            _ => unreachable!(),
        }
    }

    fn class(&mut self, marker: OpenMarker) {
        assert!(self.at(TokenKind::Class));
        self.expect(TokenKind::Class);
        self.expect(TokenKind::Identifier);
        self.parameters();
        if self.at(TokenKind::Colon) {
            self.inherits();
        }
        self.expect(TokenKind::LeftBrace);
        self.items(Some(TokenKind::RightBrace));
        self.expect(TokenKind::RightBrace);
        self.close(marker, TreeKind::Class);
    }

    fn inherits(&mut self) {
        assert!(self.at(TokenKind::Colon));
        self.expect(TokenKind::Colon);
        let marker = self.open();
        while !self.end_of_file() {
            if self.at(TokenKind::LeftBrace) {
                break;
            }
            if self.at_any(Self::ITEM_FIRST).is_some() {
                break;
            }
            if self.at_any(Self::TYPE_FIRST).is_some() {
                self.value_type();
            } else {
                self.consume_with_error("expected 'type'");
            }
        }
        self.close(marker, TreeKind::Inherits);
    }

    fn function(&mut self, marker: OpenMarker) {
        assert!(self.at_any(&[TokenKind::Constant, TokenKind::Function]).is_some());
        self.consume(TokenKind::Constant);
        self.expect(TokenKind::Function);
        self.expect(TokenKind::Identifier);
        self.parameters();
        self.expect(TokenKind::RightArrow);
        self.value_type();
        if self.at(TokenKind::LeftBrace) {
            self.block();
        } else {
            self.expect(TokenKind::Semicolon);
        }
        self.close(marker, TreeKind::Function);
    }

    const PARAMETER_RECOVERY: &'static [TokenKind] = &[TokenKind::Semicolon, TokenKind::RightArrow, TokenKind::RightParentheses, TokenKind::LeftBrace];

    fn parameters(&mut self) {
        let marker = self.open();
        while !self.end_of_file() {
            if self.at_any(Self::PARAMETER_FIRST).is_some() {
                break;
            }
            if self.at_any(Self::ITEM_FIRST).is_some() {
                break;
            }
            if self.at_any(Self::PARAMETER_FIRST).is_some() {
                self.parameter();
            } else {
                self.consume_with_error("expected 'parameter'")
            }
        }
        self.close(marker, TreeKind::Parameters);
    }

    const PARAMETER_FIRST: &'static [TokenKind] = &[TokenKind::Constant, TokenKind::Mutable, TokenKind::Identifier];

    fn parameter(&mut self) {
        assert!(self.at_any(Self::PARAMETER_FIRST).is_some());
        let marker = self.open();
        self.consume(TokenKind::Constant);
        self.consume(TokenKind::Mutable);
        self.expect(TokenKind::Identifier);
        self.expect(TokenKind::Colon);
        self.value_type();
        self.close(marker, TreeKind::Parameter);
    }

    fn field(&mut self, marker: OpenMarker) {
        let next = self.at_any(&[TokenKind::Constant, TokenKind::Field]);
        assert!(next.is_some());
        self.consume(next.unwrap());
        self.consume(TokenKind::Mutable);
        self.expect(TokenKind::Identifier);
        self.expect(TokenKind::Colon);
        self.value_type();
        if self.at(TokenKind::Equals) {
            self.initializer();
        }
        self.expect(TokenKind::Semicolon);
        self.close(marker, TreeKind::Field);
    }

    fn initializer(&mut self) {
        assert!(self.at(TokenKind::Equals));
        let marker = self.open();
        self.expect(TokenKind::Equals);
        self.expression();
        self.close(marker, TreeKind::Initializer);
    }

    const TYPE_FIRST: &'static [TokenKind] = Self::EXPRESSION_FIRST;

    fn value_type(&mut self) {
        let marker = self.open();
        self.expression();
        self.close(marker, TreeKind::Type);
    }

    const EXPRESSION_FIRST: &'static [TokenKind] = &[TokenKind::Identifier];

    fn expression(&mut self) {

    }

    fn block(&mut self) {

    }
}
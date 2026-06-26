use crate::cst::parser::event::OpenMarker;
use crate::cst::parser::Parser;
use crate::cst::token::TokenKind;
use crate::cst::tree::TreeKind;

impl<'text> Parser<'text> {
    pub fn file(&mut self) {
        let marker = self.open();
        self.items(None);
        self.close(marker, TreeKind::File);
    }

    const ITEM_FIRST: &'static [TokenKind] = &[TokenKind::Struct, TokenKind::Fn, TokenKind::Let];

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
            Some(TokenKind::Struct) => self.structure(marker),
            Some(TokenKind::Fn) => self.function(marker),
            Some(TokenKind::Let) => self.field(marker),
            _ => unreachable!(),
        }
    }

    fn structure(&mut self, marker: OpenMarker) {
        assert!(self.at(TokenKind::Struct));
        self.expect(TokenKind::Struct);
        self.expect(TokenKind::Identifier);
        self.expect(TokenKind::LeftBrace);
        self.items(Some(TokenKind::RightBrace));
        self.expect(TokenKind::RightBrace);
        self.close(marker, TreeKind::Struct);
    }

    fn function(&mut self, marker: OpenMarker) {
        assert!(self.at(TokenKind::Fn));
        self.expect(TokenKind::Fn);
        self.expect(TokenKind::Identifier);
        self.parameters();
        self.expect(TokenKind::RightArrow);
        self.value_type();
        self.block();
        self.close(marker, TreeKind::Function);
    }

    const PARAMETER_RECOVERY: &'static [TokenKind] = &[TokenKind::Semicolon, TokenKind::RightArrow, TokenKind::RightParentheses, TokenKind::LeftBrace];

    fn parameters(&mut self) {
        let marker = self.open();
        self.expect(TokenKind::LeftParentheses);
        while !self.end_of_file() {
            if self.at_any(Self::PARAMETER_RECOVERY).is_some() {
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
        self.expect(TokenKind::RightParentheses);
        self.close(marker, TreeKind::Parameters);
    }

    const PARAMETER_FIRST: &'static [TokenKind] = &[TokenKind::Identifier];

    fn parameter(&mut self) {
        assert!(self.at_any(Self::PARAMETER_FIRST).is_some());
        let marker = self.open();
        self.expect(TokenKind::Identifier);
        self.expect(TokenKind::Colon);
        self.value_type();
        self.close(marker, TreeKind::Parameter);
    }

    fn field(&mut self, marker: OpenMarker) {
        assert!(self.at(TokenKind::Let));
        self.expect(TokenKind::Let);
        self.expect(TokenKind::Identifier);
        self.expect(TokenKind::Colon);
        self.value_type();
        // For the moment we don't support creating a field without assigning it a value.
        self.initializer();
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

    const TYPE_FIRST: &'static [TokenKind] = &[TokenKind::Identifier];

    fn value_type(&mut self) {
        let marker = self.open();
        self.expect(TokenKind::Identifier);
        self.close(marker, TreeKind::Type);
    }

    const EXPRESSION_FIRST: &'static [TokenKind] = &[TokenKind::Identifier];

    fn expression(&mut self) {
        // TODO: Implement expression parsing.
        let marker = self.open();
        self.expect(TokenKind::Identifier);
        self.close(marker, TreeKind::PathExpression);
    }

    fn block(&mut self) {
        // TODO: Implement expression parsing.
        let marker = self.open();
        self.expect(TokenKind::LeftBrace);
        self.expect(TokenKind::RightBrace);
        self.close(marker, TreeKind::BlockExpression);
    }
}
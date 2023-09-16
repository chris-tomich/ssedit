use super::{
    lexer::JsonToken,
    path::{JsonPath, JsonPathOperator},
};

pub struct JsonQuery<'a> {
    path: JsonPathCursor<'a>,
    current_match_ended: bool,
    current_match_depth: isize,
}

impl<'a> JsonQuery<'a> {
    pub fn from(path: &'a JsonPath) -> JsonQuery {
        JsonQuery {
            path: JsonPathCursor::from(path),
            current_match_ended: false,
            current_match_depth: -1,
        }
    }

    pub fn parse(&mut self, token: &JsonToken) -> bool {
        let before_parse_match_state = self.path.is_matching();

        match token {
            JsonToken::PropertyName { raw: _, name } => self.path.member_access(name),
            JsonToken::ObjectOpen(_) => self.path.traverse(),
            JsonToken::ObjectClose(_) => self.path.recede(),
            JsonToken::ArrayOpen(_) => {
                self.path.traverse();
                self.path.increment_index();
            }
            JsonToken::ArrayClose(_) => self.path.recede(),
            JsonToken::ArrayItemDelimiter(_) => {
                self.path.increment_index();
            }
            _ => {}
        }

        let mut is_matching = self.path.is_matching();

        let matching_just_started = !before_parse_match_state && is_matching;

        if !is_matching && self.current_match_ended {
            self.current_match_ended = false;
        }

        if is_matching {
            match token {
                JsonToken::PropertyName { raw: _, name: _ } => {
                    if self.current_match_depth <= 0 {
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::ObjectOpen(_) => self.current_match_depth += 1,
                JsonToken::ObjectClose(_) => {
                    self.current_match_depth -= 1;

                    if self.current_match_depth == 0 {
                        self.current_match_ended = true;
                        return is_matching;
                    }
                }
                JsonToken::ArrayOpen(_) => self.current_match_depth += 1,
                JsonToken::ArrayClose(_) => {
                    self.current_match_depth -= 1;

                    if self.current_match_depth == 0 {
                        self.current_match_ended = true;
                        return is_matching;
                    }
                }
                JsonToken::ArrayItemDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        if !matching_just_started {
                            self.current_match_ended = true;
                        }
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::PropertyDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        self.current_match_ended = true;
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::KeyValueDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                _ => {}
            }
        }

        if self.current_match_ended {
            false
        } else {
            is_matching
        }
    }
}

struct JsonPathCursor<'a> {
    path: &'a JsonPath,
    path_cursor: usize,
    document_cursor: usize,
    document_array_cursors: Vec<isize>,
    path_aligned: bool,
    path_match: bool,
}

impl<'a> JsonPathCursor<'a> {
    fn from(path: &'a JsonPath) -> JsonPathCursor {
        JsonPathCursor {
            path,
            path_cursor: 0,
            document_cursor: 0,
            document_array_cursors: Vec::new(),
            path_aligned: true,
            path_match: false,
        }
    }

    fn is_array_root(&self) -> bool {
        match &self.path.operations()[self.path_cursor] {
            JsonPathOperator::ArrayRoot(_) => true,
            _ => false,
        }
    }

    fn traverse(&mut self) {
        if !self.is_array_root() && self.path_cursor != self.document_cursor {
            self.document_cursor += 1;
            return;
        } else {
            self.document_cursor += 1;

            self.document_array_cursors.push(-1);
        }

        if !self.path_aligned {
            return;
        }

        if !self.is_array_root() && self.path_cursor == self.path.operations().len() - 1 {
            return;
        }

        match &self.path.operations()[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_cursor += 1,
            JsonPathOperator::ArrayRoot(_) => {}
            JsonPathOperator::MemberAccess(_) => self.path_cursor += 1,
            JsonPathOperator::DeepScanMemberAccess(_) => todo!(),
            JsonPathOperator::ArrayIndex(_) => self.path_cursor += 1,
            JsonPathOperator::ArraySlice(_, _) => todo!(),
            JsonPathOperator::FilterExpression(_) => todo!(),
        }
    }

    fn recede(&mut self) {
        if self.path_cursor != self.document_cursor {
            self.document_cursor -= 1;
            self.document_array_cursors.pop();
            return;
        } else {
            self.path_cursor -= 1;
            self.document_cursor -= 1;
            self.document_array_cursors.pop();
            self.path_aligned = true;
            self.path_match = false;
        }
    }

    fn member_access(&mut self, name: &String) {
        if self.path_cursor != self.document_cursor {
            return;
        }

        match &self.path.operations()[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_aligned = false,
            JsonPathOperator::ArrayRoot(_) => self.path_aligned = false,
            JsonPathOperator::MemberAccess(path_member) => self.path_aligned = *name == *path_member,
            JsonPathOperator::DeepScanMemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::ArrayIndex(_) => self.path_aligned = false,
            JsonPathOperator::ArraySlice(_, _) => self.path_aligned = false,
            JsonPathOperator::FilterExpression(_) => self.path_aligned = false,
        }

        self.path_match = self.path_aligned && self.path_cursor == self.path.operations().len() - 1;
    }

    fn increment_index(&mut self) {
        if !self.is_array_root() && self.path_cursor != self.document_cursor {
            return;
        }

        match &self.path.operations()[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_aligned = false,
            JsonPathOperator::ArrayRoot(path_index) => {
                if let Some(document_array_cursor) = self.document_array_cursors.last_mut() {
                    *document_array_cursor += 1;
                    self.path_aligned = *document_array_cursor == *path_index;
                } else {
                    self.path_aligned = false
                }
            }
            JsonPathOperator::MemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::DeepScanMemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::ArrayIndex(path_index) => {
                if let Some(document_array_cursor) = self.document_array_cursors.last_mut() {
                    *document_array_cursor += 1;
                    self.path_aligned = *document_array_cursor == *path_index;
                } else {
                    self.path_aligned = false
                }
            }
            JsonPathOperator::ArraySlice(_, _) => self.path_aligned = false,
            JsonPathOperator::FilterExpression(_) => self.path_aligned = false,
        }

        self.path_match = self.path_aligned && self.path_cursor == self.path.operations().len() - 1;
    }

    fn is_matching(&self) -> bool {
        self.path_match
    }
}

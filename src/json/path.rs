use std::fmt;

pub enum JsonPathOperator {
    ObjectRoot,
    ArrayRoot(isize),
    MemberAccess(String),
    DeepScanMemberAccess(String),
    ArrayIndex(isize),
    ArraySlice(isize, isize),
    FilterExpression(String),
}

impl fmt::Display for JsonPathOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        match self {
            JsonPathOperator::ObjectRoot => {
                output.push_str("ObjectRoot");
            }
            JsonPathOperator::ArrayRoot(index) => {
                output.push_str("ArrayRoot(");
                output.push_str(index.to_string().as_str());
                output.push_str(")");
            }
            JsonPathOperator::MemberAccess(name) => {
                output.push_str("MemberAccess(");
                output.push_str(name);
                output.push_str(")");
            }
            JsonPathOperator::DeepScanMemberAccess(name) => {
                output.push_str("DeepScanMemberAccess(");
                output.push_str(name);
                output.push_str(")");
            }
            JsonPathOperator::ArrayIndex(index) => {
                output.push_str("ArrayIndex(");
                output.push_str(index.to_string().as_str());
                output.push_str(")");
            }
            JsonPathOperator::ArraySlice(start, end) => {
                output.push_str("ArraySlice(");
                output.push_str(start.to_string().as_str());
                output.push_str(",");
                output.push_str(end.to_string().as_str());
                output.push_str(")");
            }
            JsonPathOperator::FilterExpression(filter) => {
                output.push_str("FilterExpression(");
                output.push_str(filter);
                output.push_str(")");
            }
        };

        write!(f, "{}", output)
    }
}

enum JsonPathPartialOperator {
    Root,
    OpenRootBracket,
    ArrayRootIndex(String),
    PreMemberAccess,
    MemberAccess(String),
    DeepScanMemberAccess(String),
    OpenBracket,
    ArrayIndex(String),
    ArraySlice(String),
    OpenFilter,
    FilterExpression { depth: isize, expr: String },
    EscapeCharacter(),
    OpenSingleQuotes(String),
    OpenDoubleQuotes(String),
    ClosedSingleQuotes(String),
    ClosedDoubleQuotes(String),
}

pub struct JsonPathIterator<'a> {
    path_data: &'a Vec<JsonPathOperator>,
    current_index: usize,
}

impl<'a> JsonPathIterator<'a> {
    fn from(path: &JsonPath) -> JsonPathIterator {
        JsonPathIterator {
            path_data: &path.operations,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for JsonPathIterator<'a> {
    type Item = &'a JsonPathOperator;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.path_data.len() {
            let item = &self.path_data[self.current_index];
            self.current_index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct JsonPath {
    path: String,
    operations: Vec<JsonPathOperator>,
    partial_operations: Vec<JsonPathPartialOperator>,
}

impl JsonPath {
    pub fn from(path: &str) -> JsonPath {
        let mut json_path = JsonPath {
            path: String::from(path),
            operations: Vec::new(),
            partial_operations: Vec::new(),
        };
        json_path.tokenize();

        json_path
    }

    pub fn iter(&self) -> JsonPathIterator<'_> {
        JsonPathIterator::from(self)
    }

    pub fn operations(&self) -> &Vec<JsonPathOperator> {
        &self.operations
    }

    fn tokenize(&mut self) {
        let mut terminated_path = self.path.clone();
        terminated_path.push_str("\n");

        for c in terminated_path.chars() {
            match c {
                '$' => {
                    if let None = self.partial_operations.pop() {
                        self.partial_operations.push(JsonPathPartialOperator::Root);
                    } else {
                        todo!();
                    }
                }
                '.' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(String::new())),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => {
                                self.operations.push(JsonPathOperator::DeepScanMemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => todo!(),
                        }
                    } else {
                        self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                    }
                }
                '[' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => self.partial_operations.push(JsonPathPartialOperator::OpenRootBracket),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::OpenBracket);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => self.partial_operations.push(JsonPathPartialOperator::OpenBracket),
                        }
                    } else {
                        self.partial_operations.push(JsonPathPartialOperator::OpenBracket)
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => self.partial_operations.push(JsonPathPartialOperator::ArrayRootIndex(String::from(c))),
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::ArrayIndex(String::from(c))),
                            JsonPathPartialOperator::ArrayRootIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArrayRootIndex(index));
                            }
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArrayIndex(index));
                            }
                            JsonPathPartialOperator::ArraySlice(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                ']' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::ArrayRootIndex(index) => {
                                if let Ok(i) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayRoot(i));
                                }
                            }
                            JsonPathPartialOperator::ArrayIndex(index) => {
                                if let Ok(i) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayIndex(i));
                                }
                            }
                            JsonPathPartialOperator::ArraySlice(index) => {
                                let mut indexes = index.split(':');
                                let first = indexes.next();
                                let second = indexes.next();

                                if let Some(start) = first {
                                    if let Some(end) = second {
                                        if let Ok(start) = start.parse::<isize>() {
                                            if let Ok(end) = end.parse::<isize>() {
                                                self.operations.push(JsonPathOperator::ArraySlice(start, end));
                                            } else {
                                                panic!("open ended slices not supported yet");
                                            }
                                        } else {
                                            panic!("open ended slices not supported yet");
                                        }
                                    } else {
                                        panic!("open ended slices not supported yet");
                                    }
                                } else {
                                    panic!("open ended slices not supported yet");
                                }
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::ClosedDoubleQuotes(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                            }
                            _ => {}
                        }
                    }
                }
                '\'' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(name) => self.partial_operations.push(JsonPathPartialOperator::ClosedSingleQuotes(name)),
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                '"' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(String::new())),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name))
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => self.partial_operations.push(JsonPathPartialOperator::ClosedDoubleQuotes(name)),
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                '\\' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {}
                            JsonPathPartialOperator::OpenSingleQuotes(name) => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                self.partial_operations.push(JsonPathPartialOperator::EscapeCharacter());
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                self.partial_operations.push(JsonPathPartialOperator::EscapeCharacter())
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                ':' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(name));
                            }
                            JsonPathPartialOperator::Root => todo!("{}", c),
                            JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::ArraySlice(String::from(c))),
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!("{}", c),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!("{}", c),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                '?' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenFilter);
                            }
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                '(' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => todo!(),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => {
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth: 0, expr: String::new() });
                            }
                            JsonPathPartialOperator::FilterExpression { mut depth, mut expr } => {
                                expr.push(c);
                                depth += 1;
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                ')' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => todo!(),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!(),
                            JsonPathPartialOperator::FilterExpression { mut depth, mut expr } => {
                                if depth > 0 {
                                    expr.push(c);
                                    depth -= 1;
                                    self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr });
                                } else {
                                    self.operations.push(JsonPathOperator::FilterExpression(expr));
                                }
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    }
                }
                '\n' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => self.operations.push(JsonPathOperator::ObjectRoot),
                            JsonPathPartialOperator::OpenRootBracket => self.operations.push(JsonPathOperator::ArrayRoot(-1)),
                            JsonPathPartialOperator::ArrayRootIndex(index) => {
                                if let Ok(index) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayRoot(index));
                                }
                            }
                            JsonPathPartialOperator::PreMemberAccess => {}
                            JsonPathPartialOperator::MemberAccess(name) => self.operations.push(JsonPathOperator::MemberAccess(name)),
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => self.operations.push(JsonPathOperator::DeepScanMemberAccess(name)),
                            JsonPathPartialOperator::OpenBracket => self.operations.push(JsonPathOperator::MemberAccess(String::new())),
                            JsonPathPartialOperator::ArrayIndex(index) => {
                                if let Ok(index) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayIndex(index));
                                }
                            }
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!(),
                            JsonPathPartialOperator::FilterExpression { depth: _, expr: _ } => {
                                todo!()
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::OpenDoubleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    }
                }
                _ => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(name));
                            }
                            JsonPathPartialOperator::Root => todo!("{}", c),
                            JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!("{}", c),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!("{}", c),
                        }
                    } else {
                        todo!("[{}]", c);
                    }
                }
            }
        }
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        let mut is_first = true;

        for operation in &self.operations {
            if !is_first {
                output.push_str(" -> ")
            } else {
                is_first = false;
            }

            output.push_str(operation.to_string().as_str());
        }

        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_path_object_root() {
        assert_eq!(JsonPath::from("$.").to_string(), "ObjectRoot");
    }

    #[test]
    fn test_json_path_array_root() {
        assert_eq!(JsonPath::from("$[5]").to_string(), "ArrayRoot(5)");
    }
}

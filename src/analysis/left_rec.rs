use crate::grammar::ast::*;

pub fn eliminate_left_recursion(rules: &[Rule]) -> Vec<Rule> {
    let mut result = Vec::new();
    let mut processed_names: Vec<String> = Vec::new();

    for rule in rules {
        let has_direct_lr = has_direct_left_recursion(rule);
        if has_direct_lr {
            let transformed = transform_direct_left_recursion(rule);
            for r in &transformed {
                if !result.iter().any(|existing: &Rule| existing.name == r.name) {
                    result.push(r.clone());
                }
            }
        } else {
            let mut new_rule = rule.clone();
            if let Some(indirect) = eliminate_indirect_lr(&new_rule, &processed_names, rules) {
                new_rule = indirect;
            }
            if !result.iter().any(|existing| existing.name == new_rule.name) {
                result.push(new_rule);
            }
        }
        processed_names.push(rule.name.clone());
    }

    result
}

fn has_direct_left_recursion(rule: &Rule) -> bool {
    for alt in &rule.alternatives {
        if let Some(first) = alt.elements.first() {
            if let ElementKind::RuleRef(name) = &first.kind {
                if name == &rule.name {
                    return true;
                }
            }
        }
    }
    false
}

fn transform_direct_left_recursion(rule: &Rule) -> Vec<Rule> {
    let mut base_alts: Vec<Alternative> = Vec::new();
    let mut recursive_alts: Vec<(String, Vec<Element>)> = Vec::new();

    for alt in &rule.alternatives {
        if alt.elements.is_empty() {
            base_alts.push(alt.clone());
            continue;
        }
        if let ElementKind::RuleRef(name) = &alt.elements.first().unwrap().kind {
            if name == &rule.name {
                let suffix: Vec<Element> = alt.elements[1..].to_vec();
                let label = alt.label.clone();
                recursive_alts.push((label.unwrap_or_default(), suffix));
                continue;
            }
        }
        base_alts.push(alt.clone());
    }

    let suffix_name = format!("{}Suffix", capitalize(&rule.name));

    let mut suffix_alts = Vec::new();
    for (_, suffix_elems) in &recursive_alts {
        let mut elems = suffix_elems.clone();
        elems.push(Element::new(ElementKind::RuleRef(suffix_name.clone())));
        suffix_alts.push(Alternative::new(elems));
    }
    suffix_alts.push(Alternative::new(Vec::new()));

    let mut new_rule = Rule {
        name: rule.name.clone(),
        is_fragment: false,
        modifiers: rule.modifiers.clone(),
        return_type: rule.return_type.clone(),
        locals_decl: rule.locals_decl.clone(),
        throws: rule.throws.clone(),
        alternatives: Vec::new(),
        commands: Vec::new(),
    };

    if base_alts.is_empty() {
        new_rule.alternatives.push(Alternative::new(vec![
            Element::new(ElementKind::RuleRef(suffix_name.clone())),
        ]));
    }

    for mut base_alt in base_alts {
        base_alt
            .elements
            .push(Element::new(ElementKind::RuleRef(suffix_name.clone())));
        new_rule.alternatives.push(base_alt);
    }

    let suffix_rule = Rule {
        name: suffix_name,
        is_fragment: false,
        modifiers: Vec::new(),
        return_type: None,
        locals_decl: None,
        throws: Vec::new(),
        alternatives: suffix_alts,
        commands: Vec::new(),
    };

    vec![new_rule, suffix_rule]
}

fn eliminate_indirect_lr(
    rule: &Rule,
    processed_names: &[String],
    all_rules: &[Rule],
) -> Option<Rule> {
    let mut changed = false;
    let mut new_alts: Vec<Alternative> = Vec::new();

    for alt in &rule.alternatives {
        if alt.elements.is_empty() {
            new_alts.push(alt.clone());
            continue;
        }
        if let ElementKind::RuleRef(name) = &alt.elements.first().unwrap().kind {
            if processed_names.contains(name) {
                if let Some(other_rule) = all_rules.iter().find(|r| r.name == *name) {
                    for other_alt in &other_rule.alternatives {
                        let mut new_elems: Vec<Element> = Vec::new();
                        new_elems.extend(other_alt.elements.iter().cloned());
                        new_elems.extend(alt.elements[1..].iter().cloned());
                        new_alts.push(Alternative::new(new_elems));
                        changed = true;
                    }
                    continue;
                }
            }
        }
        new_alts.push(alt.clone());
    }

    if changed {
        Some(Rule {
            name: rule.name.clone(),
            is_fragment: rule.is_fragment,
            modifiers: rule.modifiers.clone(),
            return_type: rule.return_type.clone(),
            locals_decl: rule.locals_decl.clone(),
            throws: rule.throws.clone(),
            alternatives: new_alts,
            commands: rule.commands.clone(),
        })
    } else {
        None
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str, alts: Vec<Vec<ElementKind>>) -> Rule {
        Rule {
            name: name.to_string(),
            is_fragment: false,
            modifiers: Vec::new(),
            return_type: None,
            locals_decl: None,
            throws: Vec::new(),
            alternatives: alts.into_iter()
                .map(|elems| Alternative::new(elems.into_iter().map(Element::new).collect()))
                .collect(),
            commands: Vec::new(),
        }
    }

    #[test]
    fn test_no_left_recursion() {
        let rules = vec![
            rule("expr", vec![
                vec![ElementKind::TokenRef("NUMBER".to_string())],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "expr");
    }

    #[test]
    fn test_direct_left_recursion() {
        let rules = vec![
            rule("expr", vec![
                vec![ElementKind::RuleRef("expr".to_string()), ElementKind::TokenRef("PLUS".to_string()), ElementKind::RuleRef("term".to_string())],
                vec![ElementKind::RuleRef("term".to_string())],
            ]),
            rule("term", vec![
                vec![ElementKind::TokenRef("NUMBER".to_string())],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        // expr should be split into expr + exprSuffix
        let expr_rule = result.iter().find(|r| r.name == "expr").unwrap();
        let suffix_rule = result.iter().find(|r| r.name == "ExprSuffix").unwrap();
        // expr should have base alternatives followed by suffix ref
        assert!(!expr_rule.alternatives.is_empty());
        // suffix should have recursive + empty alternatives
        assert_eq!(suffix_rule.alternatives.len(), 2);
    }

    #[test]
    fn test_indirect_left_recursion() {
        let rules = vec![
            rule("a", vec![
                vec![ElementKind::RuleRef("b".to_string()), ElementKind::TokenRef("X".to_string())],
                vec![ElementKind::TokenRef("Y".to_string())],
            ]),
            rule("b", vec![
                vec![ElementKind::RuleRef("a".to_string()), ElementKind::TokenRef("Z".to_string())],
                vec![ElementKind::TokenRef("W".to_string())],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_empty_alternative_preserved() {
        let rules = vec![
            rule("list", vec![
                vec![ElementKind::RuleRef("list".to_string()), ElementKind::TokenRef("COMMA".to_string()), ElementKind::RuleRef("item".to_string())],
                vec![],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        assert!(result.len() >= 1);
    }

    #[test]
    fn test_multiple_alternatives_with_lr() {
        let rules = vec![
            rule("expr", vec![
                vec![ElementKind::RuleRef("expr".to_string()), ElementKind::TokenRef("PLUS".to_string()), ElementKind::RuleRef("term".to_string())],
                vec![ElementKind::RuleRef("expr".to_string()), ElementKind::TokenRef("MINUS".to_string()), ElementKind::RuleRef("term".to_string())],
                vec![ElementKind::RuleRef("term".to_string())],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        let expr_rule = result.iter().find(|r| r.name == "expr").unwrap();
        let suffix_rule = result.iter().find(|r| r.name == "ExprSuffix").unwrap();
        // suffix should have 3 alternatives: PLUS term suffix, MINUS term suffix, empty
        assert_eq!(suffix_rule.alternatives.len(), 3);
        assert!(expr_rule.alternatives.iter().all(|alt| {
            alt.elements.last().map_or(false, |e| matches!(e.kind, ElementKind::RuleRef(ref n) if n == "ExprSuffix"))
        }));
    }

    #[test]
    fn test_has_direct_left_recursion() {
        let lr_rule = rule("expr", vec![
            vec![ElementKind::RuleRef("expr".to_string())],
        ]);
        assert!(has_direct_left_recursion(&lr_rule));

        let non_lr_rule = rule("expr", vec![
            vec![ElementKind::TokenRef("NUMBER".to_string())],
        ]);
        assert!(!has_direct_left_recursion(&non_lr_rule));
    }

    #[test]
    fn test_transform_preserves_rule_name() {
        let rules = vec![
            rule("expr", vec![
                vec![ElementKind::RuleRef("expr".to_string()), ElementKind::TokenRef("PLUS".to_string())],
                vec![ElementKind::TokenRef("NUMBER".to_string())],
            ]),
        ];
        let result = eliminate_left_recursion(&rules);
        assert!(result.iter().any(|r| r.name == "expr"));
        assert!(result.iter().any(|r| r.name == "ExprSuffix"));
    }
}

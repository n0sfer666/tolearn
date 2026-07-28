#[derive(Debug, Default, Clone)]
pub struct Robots {
    rules: Vec<(String, bool)>,
}

impl Robots {
    pub fn read(text: &str) -> Self {
        let mut rules = Vec::new();
        let mut ours = false;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or_default().trim();
            let Some((field, value)) = line.split_once(':') else {
                continue;
            };
            let field = field.trim().to_ascii_lowercase();
            let value = value.trim();
            match field.as_str() {
                "user-agent" => ours = value == "*",
                "disallow" if ours && !value.is_empty() => {
                    rules.push((value.to_string(), false));
                }
                "allow" if ours && !value.is_empty() => rules.push((value.to_string(), true)),
                _ => {}
            }
        }
        Self { rules }
    }

    pub fn allows(&self, path: &str) -> bool {
        self.rules
            .iter()
            .filter(|(prefix, _)| path.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .is_none_or(|(_, allowed)| *allowed)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "offline gate: a panic here is the report"
)]
mod tests {
    use super::Robots;

    #[test]
    fn пустой_файл_разрешает_всё() {
        assert!(Robots::read("").allows("/anything"));
    }

    #[test]
    fn правила_чужого_агента_не_наши() {
        let robots = Robots::read("User-agent: Googlebot\nDisallow: /\n");

        assert!(robots.allows("/anything"));
    }

    #[test]
    fn длинное_правило_побеждает_короткое() {
        let robots = Robots::read("User-agent: *\nDisallow: /a/\nAllow: /a/b\n");

        assert!(!robots.allows("/a/x"));
        assert!(robots.allows("/a/b"));
    }

    #[test]
    fn пустой_disallow_ничего_не_запрещает() {
        let robots = Robots::read("User-agent: *\nDisallow:\n");

        assert!(robots.allows("/anything"));
    }

    #[test]
    fn комментарий_не_путает_разбор() {
        let robots = Robots::read("User-agent: * # все\nDisallow: /private/ # закрыто\n");

        assert!(!robots.allows("/private/x"));
    }
}

use tolearn_core::places::places;

#[test]
fn настройки_лежат_в_конфиге_а_хранилище_в_данных() {
    let root = std::env::temp_dir().join(format!("tolearn-places-{}", std::process::id()));

    let made = places(&root);

    if std::env::var_os("XDG_CONFIG_HOME").is_none() {
        assert_eq!(made.config, root.join(".config/tolearn"));
    }
    if std::env::var_os("XDG_DATA_HOME").is_none() {
        assert_eq!(made.data, root.join(".local/share/tolearn"));
    }
    assert_ne!(made.config, made.data);
}

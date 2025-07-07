use derive_macro::Merge;
use traxor::merge::Merge;

#[derive(Debug, PartialEq, Merge)]
struct NestedStruct {
    value: u32,
    name: Option<String>,
}

#[derive(Debug, PartialEq, Merge)]
struct TestStruct {
    a: u32,
    b: Option<String>,
    c: Option<bool>,
    nested: NestedStruct,
    optional_nested: Option<NestedStruct>,
}

#[test]
fn test_merge_basic_fields() {
    let mut s1 = TestStruct {
        a: 1,
        b: Some("hello".to_string()),
        c: None,
        nested: NestedStruct {
            value: 10,
            name: Some("original".to_string()),
        },
        optional_nested: None,
    };
    let s2 = TestStruct {
        a: 2,
        b: Some("world".to_string()),
        c: Some(true),
        nested: NestedStruct {
            value: 20,
            name: Some("new".to_string()),
        },
        optional_nested: Some(NestedStruct {
            value: 30,
            name: Some("optional".to_string()),
        }),
    };

    s1.merge(s2);

    assert_eq!(s1.a, 2);
    assert_eq!(s1.b, Some("world".to_string()));
    assert_eq!(s1.c, Some(true));
    assert_eq!(s1.nested.value, 20);
    assert_eq!(s1.nested.name, Some("new".to_string()));
    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 30,
            name: Some("optional".to_string())
        })
    );
}

#[test]
fn test_merge_option_none_other() {
    let mut s1 = TestStruct {
        a: 1,
        b: Some("hello".to_string()),
        c: Some(false),
        nested: NestedStruct {
            value: 10,
            name: Some("original".to_string()),
        },
        optional_nested: Some(NestedStruct {
            value: 100,
            name: Some("existing".to_string()),
        }),
    };
    let s2 = TestStruct {
        a: 2,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 20,
            name: None,
        },
        optional_nested: None,
    };

    s1.merge(s2);

    assert_eq!(s1.a, 2);
    assert_eq!(s1.b, Some("hello".to_string())); // Should remain Some("hello")
    assert_eq!(s1.c, Some(false)); // Should remain Some(false)
    assert_eq!(s1.nested.value, 20);
    assert_eq!(s1.nested.name, Some("original".to_string())); // Should remain original
    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 100,
            name: Some("existing".to_string())
        })
    ); // Should remain existing
}

#[test]
fn test_merge_option_some_other() {
    let mut s1 = TestStruct {
        a: 1,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 10,
            name: None,
        },
        optional_nested: None,
    };
    let s2 = TestStruct {
        a: 2,
        b: Some("world".to_string()),
        c: Some(true),
        nested: NestedStruct {
            value: 20,
            name: Some("new".to_string()),
        },
        optional_nested: Some(NestedStruct {
            value: 30,
            name: Some("optional".to_string()),
        }),
    };

    s1.merge(s2);

    assert_eq!(s1.a, 2);
    assert_eq!(s1.b, Some("world".to_string()));
    assert_eq!(s1.c, Some(true));
    assert_eq!(s1.nested.value, 20);
    assert_eq!(s1.nested.name, Some("new".to_string()));
    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 30,
            name: Some("optional".to_string())
        })
    );
}

#[test]
fn test_merge_nested_struct_with_none_name() {
    let mut s1 = TestStruct {
        a: 1,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 10,
            name: Some("original".to_string()),
        },
        optional_nested: None,
    };
    let s2 = TestStruct {
        a: 2,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 20,
            name: None,
        },
        optional_nested: None,
    };

    s1.merge(s2);

    assert_eq!(s1.nested.value, 20);
    assert_eq!(s1.nested.name, Some("original".to_string())); // Should remain original
}

#[test]
fn test_merge_optional_nested_struct_some_to_some() {
    let mut s1 = TestStruct {
        a: 1,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 10,
            name: None,
        },
        optional_nested: Some(NestedStruct {
            value: 100,
            name: Some("existing".to_string()),
        }),
    };
    let s2 = TestStruct {
        a: 2,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 20,
            name: None,
        },
        optional_nested: Some(NestedStruct {
            value: 200,
            name: Some("new_optional".to_string()),
        }),
    };

    s1.merge(s2);

    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 200,
            name: Some("new_optional".to_string())
        })
    );
}

#[test]
fn test_merge_optional_nested_struct_none_to_some() {
    let mut s1 = TestStruct {
        a: 1,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 10,
            name: None,
        },
        optional_nested: None,
    };
    let s2 = TestStruct {
        a: 2,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 20,
            name: None,
        },
        optional_nested: Some(NestedStruct {
            value: 200,
            name: Some("new_optional".to_string()),
        }),
    };

    s1.merge(s2);

    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 200,
            name: Some("new_optional".to_string())
        })
    );
}

#[test]
fn test_merge_optional_nested_struct_some_to_none() {
    let mut s1 = TestStruct {
        a: 1,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 10,
            name: None,
        },
        optional_nested: Some(NestedStruct {
            value: 100,
            name: Some("existing".to_string()),
        }),
    };
    let s2 = TestStruct {
        a: 2,
        b: None,
        c: None,
        nested: NestedStruct {
            value: 20,
            name: None,
        },
        optional_nested: None,
    };

    s1.merge(s2);

    assert_eq!(
        s1.optional_nested,
        Some(NestedStruct {
            value: 100,
            name: Some("existing".to_string())
        })
    ); // Should remain existing
}

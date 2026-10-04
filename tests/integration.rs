use serde_json::json;
use shae_morph::transform;

#[test]
fn test_shae_basic_object_transform() {
    let script = r#"
        out Profile = {
            full_name: .user.name,
            account_age: .user.meta.age ?? 21,
            is_active: .user.is_active ?? false
        }
    "#;

    let input = json!({
        "user": {
            "name": "Satya",
            "is_active": true
        }
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!({
            "full_name": "Satya",
            "account_age": 21.0,
            "is_active": true
        })
    );
}

#[test]
fn test_shae_methods_and_string_concat() {
    let script = r#"
        out CleanedUser = {
            display: .user.first_name.capitalize() + " " + .user.last_name.capitalize(),
            handle: "@" + .user.username.trim().lowercase(),
            age: .user.raw_age.toInt() ?? 0,
            tags: .user.hobbies.join(", ")
        }
    "#;

    let input = json!({
        "user": {
            "first_name": "satya",
            "last_name": "prakash",
            "username": "  SpDeDSeC  ",
            "raw_age": "21",
            "hobbies": ["coding", "compilers", "design"]
        }
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!({
            "display": "Satya Prakash",
            "handle": "@spdedsec",
            "age": 21,
            "tags": "coding, compilers, design"
        })
    );
}

#[test]
fn test_shae_array_filtering_with_where() {
    let script = r#"
        out EligibleVoters = from .citizens[] where .age >= 18 and .registered == true select {
            voter_id: .id,
            name: .name.uppercase()
        }
    "#;

    let input = json!({
        "citizens": [
            { "id": "v1", "name": "alice", "age": 17, "registered": true },
            { "id": "v2", "name": "bob", "age": 25, "registered": true },
            { "id": "v3", "name": "charlie", "age": 30, "registered": false },
            { "id": "v4", "name": "satya", "age": 21, "registered": true }
        ]
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!([
            { "voter_id": "v2", "name": "BOB" },
            { "voter_id": "v4", "name": "SATYA" }
        ])
    );
}

#[test]
fn test_shae_array_methods_len_first_last() {
    let script = r#"
        out Stats = {
            total_items: .items.len(),
            first_item: .items.first(),
            last_item: .items.last()
        }
    "#;

    let input = json!({
        "items": ["apple", "banana", "cherry"]
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!({
            "total_items": 3.0,
            "first_item": "apple",
            "last_item": "cherry"
        })
    );
}

#[test]
fn test_shae_missing_nested_fields_do_not_crash() {
    let script = r#"
        out SafeData = {
            id: .data.id,
            city: .data.address.geo.city ?? "Unknown City",
            zip: .data.address.zip ?? "00000"
        }
    "#;

    let input = json!({
        "data": {
            "id": "12345"
        }
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!({
            "id": "12345",
            "city": "Unknown City",
            "zip": "00000"
        })
    );
}

use serde_json::json;
use shae::transform;

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
fn test_shae_missing_nested_fields_do_not_crash() {
    let script = r#"
        out SafeData = {
            id: .data.id,
            city: .data.address.geo.city ?? "Unknown City",
            zip: .data.address.zip ?? "00000"
        }
    "#;

    // input with empty data
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

#[test]
fn test_shae_array_iteration_mapping() {
    let script = r#"
        out MappedUsers = from .users[] select {
            user_id: .id,
            display_name: .profile.nickname ?? .profile.name ?? "Anonymous"
        }
    "#;

    let input = json!({
        "users": [
            {
                "id": "u1",
                "profile": {
                    "nickname": "spdedsec",
                    "name": "Satya"
                }
            },
            {
                "id": "u2",
                "profile": {
                    "name": "Bob"
                }
            },
            {
                "id": "u3"
            }
        ]
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!([
            { "user_id": "u1", "display_name": "spdedsec" },
            { "user_id": "u2", "display_name": "Bob" },
            { "user_id": "u3", "display_name": "Anonymous" }
        ])
    );
}

#[test]
fn test_shae_null_coalescing_semantics() {
    let script = r#"
        out Normalized = {
            email: .user.email ?? "no-email@shae.dev",
            score: .user.score ?? 100
        }
    "#;

    // Explicit null in incoming payload
    let input = json!({
        "user": {
            "email": null,
            "score": null
        }
    });

    let result = transform(script, &input).expect("Transform failed");

    assert_eq!(
        result,
        json!({
            "email": "no-email@shae.dev",
            "score": 100.0
        })
    );
}

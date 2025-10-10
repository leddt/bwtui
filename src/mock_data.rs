use crate::types::{VaultItem, ItemType, LoginData, Uri};
use chrono::Utc;

/// Generate mock vault data for the prototype
pub fn generate_mock_data() -> Vec<VaultItem> {
    vec![
        VaultItem {
            id: "1".to_string(),
            name: "GitHub".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("john.doe@example.com".to_string()),
                password: Some("super_secure_password_123".to_string()),
                totp: Some("JBSWY3DPEHPK3PXP".to_string()),
                uris: Some(vec![Uri {
                    value: "https://github.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Main GitHub account for work projects".to_string()),
            favorite: true,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "2".to_string(),
            name: "AWS Console".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("admin@company.com".to_string()),
                password: Some("aws_complex_pass_456".to_string()),
                totp: Some("JBSWY3DPEHPK3PXQ".to_string()),
                uris: Some(vec![Uri {
                    value: "https://console.aws.amazon.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Production AWS account - handle with care!".to_string()),
            favorite: true,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "3".to_string(),
            name: "Gmail".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("john.doe@gmail.com".to_string()),
                password: Some("gmail_secure_789".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://mail.google.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Personal email account".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "4".to_string(),
            name: "Digital Ocean".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("johndoe".to_string()),
                password: Some("do_password_xyz".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://cloud.digitalocean.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Hosting for side projects".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "5".to_string(),
            name: "Twitter".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("@johndoe".to_string()),
                password: Some("twitter_pass_abc".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://twitter.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: None,
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "6".to_string(),
            name: "LinkedIn".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("john.doe@example.com".to_string()),
                password: Some("linkedin_pass_def".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://linkedin.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Professional networking".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "7".to_string(),
            name: "GitLab".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("john.doe".to_string()),
                password: Some("gitlab_secure_ghi".to_string()),
                totp: Some("JBSWY3DPEHPK3PXR".to_string()),
                uris: Some(vec![Uri {
                    value: "https://gitlab.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Alternative git hosting".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "8".to_string(),
            name: "Docker Hub".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("johndoe".to_string()),
                password: Some("docker_pass_jkl".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://hub.docker.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Container registry".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "9".to_string(),
            name: "Stripe Dashboard".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("payments@company.com".to_string()),
                password: Some("stripe_secure_mno".to_string()),
                totp: Some("JBSWY3DPEHPK3PXS".to_string()),
                uris: Some(vec![Uri {
                    value: "https://dashboard.stripe.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Payment processing dashboard".to_string()),
            favorite: true,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
        VaultItem {
            id: "10".to_string(),
            name: "Slack Workspace".to_string(),
            item_type: ItemType::Login,
            login: Some(LoginData {
                username: Some("john.doe@company.com".to_string()),
                password: Some("slack_pass_pqr".to_string()),
                totp: None,
                uris: Some(vec![Uri {
                    value: "https://company.slack.com".to_string(),
                    match_type: Some(0),
                }]),
            }),
            notes: Some("Team communication".to_string()),
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: Utc::now(),
        },
    ]
}


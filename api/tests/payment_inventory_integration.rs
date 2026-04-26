use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::Client;
use serde_json::{json, Value};

fn base_url() -> String {
    std::env::var("GARAGE360_API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

fn test_email() -> String {
    std::env::var("GARAGE360_TEST_EMAIL").unwrap_or_else(|_| "admin@demo.com".to_string())
}

fn test_password() -> String {
    std::env::var("GARAGE360_TEST_PASSWORD").unwrap_or_else(|_| "admin123".to_string())
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    nanos.to_string()
}

async fn login(client: &Client) -> String {
    let response = client
        .post(format!("{}/api/v1/auth/login", base_url()))
        .json(&json!({
            "email": test_email(),
            "password": test_password()
        }))
        .send()
        .await
        .expect("login request should complete");

    assert!(
        response.status().is_success(),
        "login failed with status {}",
        response.status()
    );

    let body: Value = response.json().await.expect("login response should be json");
    body.get("access_token")
        .and_then(Value::as_str)
        .expect("access token should be present")
        .to_string()
}

#[tokio::test]
async fn inventory_adjustment_and_invoice_payment_flow() {
    let client = Client::new();
    let token = login(&client).await;
    let suffix = unique_suffix();

    let inventory_create = client
        .post(format!("{}/api/v1/inventory", base_url()))
        .bearer_auth(&token)
        .json(&json!({
            "sku": format!("IT-{}", suffix),
            "name": "Integration Test Item",
            "description": "Created by integration test",
            "category": "Test",
            "unit": "pcs",
            "cost_price": "100.00",
            "sell_price": "150.00",
            "min_stock_level": 1
        }))
        .send()
        .await
        .expect("inventory create request should complete");

    assert!(
        inventory_create.status().is_success(),
        "inventory create failed with status {}",
        inventory_create.status()
    );
    let inventory_body: Value = inventory_create
        .json()
        .await
        .expect("inventory create response should be json");
    let inventory_id = inventory_body
        .get("id")
        .and_then(Value::as_str)
        .expect("inventory id should be present")
        .to_string();

    let adjust = client
        .post(format!("{}/api/v1/inventory/{}/adjust", base_url(), inventory_id))
        .bearer_auth(&token)
        .json(&json!({
            "adjustment_type": "ADD",
            "quantity": "5.000",
            "reason": "integration test stock add"
        }))
        .send()
        .await
        .expect("inventory adjust request should complete");

    assert!(
        adjust.status().is_success(),
        "inventory adjust failed with status {}",
        adjust.status()
    );
    let adjust_body: Value = adjust.json().await.expect("inventory adjust response should be json");
    let current_quantity = adjust_body
        .get("item")
        .and_then(|item| item.get("currentQuantity"))
        .and_then(Value::as_str)
        .expect("currentQuantity should be present after adjustment");
    assert_eq!(current_quantity, "5.000");

    let customer_create = client
        .post(format!("{}/api/v1/customers", base_url()))
        .bearer_auth(&token)
        .json(&json!({
            "customer_type": "INDIVIDUAL",
            "first_name": format!("Pay{}", &suffix),
            "last_name": "Flow",
            "email": format!("pay-flow-{}@example.com", suffix),
            "phone": format!("98{}", &suffix[suffix.len().saturating_sub(8)..]),
            "address": "Integration Test Address"
        }))
        .send()
        .await
        .expect("customer create request should complete");

    assert!(
        customer_create.status().is_success(),
        "customer create failed with status {}",
        customer_create.status()
    );
    let customer_body: Value = customer_create
        .json()
        .await
        .expect("customer create response should be json");
    let customer_id = customer_body
        .get("id")
        .and_then(Value::as_str)
        .expect("customer id should be present")
        .to_string();

    let invoice_create = client
        .post(format!("{}/api/v1/invoices", base_url()))
        .bearer_auth(&token)
        .json(&json!({
            "customerId": customer_id,
            "notes": "integration test invoice",
            "lineItems": [
                {
                    "description": "Integration labor",
                    "quantity": 1,
                    "unitPrice": 2000,
                    "discountPct": 0
                }
            ]
        }))
        .send()
        .await
        .expect("invoice create request should complete");

    assert!(
        invoice_create.status().is_success(),
        "invoice create failed with status {}",
        invoice_create.status()
    );
    let invoice_body: Value = invoice_create
        .json()
        .await
        .expect("invoice create response should be json");
    let invoice_id = invoice_body
        .get("id")
        .and_then(Value::as_str)
        .expect("invoice id should be present")
        .to_string();

    let payment = client
        .post(format!("{}/api/v1/invoices/{}/payment", base_url(), invoice_id))
        .bearer_auth(&token)
        .json(&json!({
            "amount": 500,
            "paymentMethod": "cash",
            "paymentRef": "  IT-PAY-REF  ",
            "notes": "integration payment"
        }))
        .send()
        .await
        .expect("invoice payment request should complete");

    assert!(
        payment.status().is_success(),
        "invoice payment failed with status {}",
        payment.status()
    );
    let payment_body: Value = payment
        .json()
        .await
        .expect("invoice payment response should be json");
    let amount_paid = payment_body
        .get("amountPaid")
        .and_then(Value::as_str)
        .expect("amountPaid should be present");
    assert_eq!(amount_paid, "500.00");
    let payment_method = payment_body
        .get("paymentMethod")
        .and_then(Value::as_str)
        .expect("paymentMethod should be present");
    assert_eq!(payment_method, "CASH");
}

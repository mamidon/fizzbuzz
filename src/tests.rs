use crate::{read_transactions_from_text, Money};

fn test_case(text: &str) -> String {
    read_transactions_from_text(text).unwrap()
}

fn from_parts(whole: u32, decimal: u16) -> Money {
    assert!(decimal < 10000);

    Money(whole as u64 * 10000 + decimal as u64)
}

// This is not a sufficient amount of testing for implementing your own fixed point math
// I should probably have used a library
// In this case I'm not worrying about sign, as we're not allowing that to happen
// but if we were to do this for real it'd be something worth considering
#[test]
fn money_parses_whole_part_correctly() {
    let actual: Money = "3.14".parse().unwrap();

    assert_eq!(actual, from_parts(3, 1400));
}

#[test]
fn money_adds_correctly() {
    let a: Money = "3.14".parse().unwrap();
    let b: Money = "1.23".parse().unwrap();

    assert_eq!(a + b, from_parts(4, 3700));
}

#[test]
fn money_subs_correctly() {
    let a: Money = "3.14".parse().unwrap();
    let b: Money = "1.23".parse().unwrap();

    assert_eq!(a - b, from_parts(1, 9100));
}

#[test]
fn given_test_case() {
    let output = test_case(
        "\
type,       client, tx, amount
deposit,    1,      1,  1.0
deposit,    2,      2,  2.0
deposit,    1,      3,  2.0
withdrawal, 1,      4,  1.5
withdrawal, 2,      5,  3.0",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
"
    );
}

#[test]
fn simple_happy_path() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,42.0,0.0,42.0,false
"
    );
}

#[test]
fn deposits_are_commutative() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 1, 2, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,47.0,0.0,47.0,false
"
    );
}

#[test]
fn deposits_across_accounts_are_independent() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 2, 2, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,42.0,0.0,42.0,false
2,5.0,0.0,5.0,false
"
    );
}

#[test]
fn deposits_replays_are_ignored() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 1, 1, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,42.0,0.0,42.0,false
"
    );
}

#[test]
fn withdrawals_deduct() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    withdrawal, 1, 2, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,37.0,0.0,37.0,false
"
    );
}

#[test]
fn withdrawals_accross_accounts_are_indepdendent() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 2, 3, 20
    withdrawal, 1, 2, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,37.0,0.0,37.0,false
2,20.0,0.0,20.0,false
"
    );
}

#[test]
fn withdrawal_replays_are_ignored() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 2, 3, 20
    withdrawal, 1, 2, 5
    withdrawal, 1, 2, 5",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,37.0,0.0,37.0,false
2,20.0,0.0,20.0,false
"
    );
}

#[test]
fn withdrawals_are_limited_to_available_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 2, 3, 20
    withdrawal, 1, 2, 40
    withdrawal, 1, 4, 50",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,2.0,0.0,2.0,false
2,20.0,0.0,20.0,false
"
    );
}

#[test]
fn disputes_hold_relevant_tx_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 1, 3, 20
    dispute, 1, 1, 0
    withdrawal, 1, 4, 50",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,20.0,42.0,62.0,false
"
    );
}

#[test]
fn disputes_hold_only_available_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    deposit, 1, 3, 20
    withdrawal, 1, 2, 30
    dispute, 1, 1
    withdrawal, 1, 4, 32",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,0.0,32.0,32.0,false
"
    );
}

#[test]
fn dispute_replays_are_ignored() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    dispute, 1, 1, 0
    dispute, 1, 1, 0",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,0.0,42.0,42.0,false
"
    );
}

#[test]
fn resolve_releases_relevant_tx_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    dispute, 1, 1
    resolve, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,42.0,0.0,42.0,false
"
    );
}

#[test]
fn resolve_only_releases_held_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    withdrawal, 1, 2, 10
    dispute, 1, 1,
    resolve, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,32.0,0.0,32.0,false
"
    );
}

#[test]
fn resolve_only_releases_disputed_transactions() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    withdrawal, 1, 2, 10
    resolve, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,32.0,0.0,32.0,false
"
    );
}

#[test]
fn chargeback_releases_relevant_tx_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    dispute, 1, 1,
    chargeback, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,42.0,0.0,42.0,true
"
    );
}

#[test]
fn chargeback_only_releases_held_funds() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    withdrawal, 1, 2, 10
    dispute, 1, 1,
    chargeback, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,32.0,0.0,32.0,true
"
    );
}

#[test]
fn chargeback_only_releases_disputed_transactions() {
    let output = test_case(
        "\
    type, client, tx, amount
    deposit, 1, 1, 42
    withdrawal, 1, 2, 10
    chargeback, 1, 1,",
    );

    assert_eq!(
        output,
        "\
client_id,available,held,total,locked
1,32.0,0.0,32.0,false
"
    );
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{anyhow, Result};
use diem_types::account_address::AccountAddress;
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use std::{cmp::max, io, io::Write, thread, time};
use url::Url;

const DIEM_ACCOUNT_TYPE: &str = "0x1::DiemAccount::DiemAccount";

// Will list the last 10 transactions and has the ability to block and stream future transactions.
pub async fn handle(network: Url, tail: bool, address: AccountAddress, raw: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let account_seq_num = get_account_sequence_number(&client, &network, address).await?;
    let mut prev_seq_num = max(account_seq_num as i64 - 10, 0);
    let resp =
        get_account_transactions_response(&client, address, &network, prev_seq_num, 10).await?;
    let json_with_txns: serde_json::Value = serde_json::from_str(resp.text().await?.as_str())?;

    let all_transactions = json_with_txns
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Failed to get transactions"))?;

    write_out_txns(all_transactions.to_vec(), &mut io::stdout(), raw)?;

    if !all_transactions.is_empty() {
        prev_seq_num = parse_txn_for_seq_num(
            all_transactions
                .last()
                .ok_or_else(|| anyhow!("Couldn't get last transaction"))?,
        )?
    } else {
        // Setting to -1 to handle the case where sequence number is 0 and
        // we don't have a previous sequence number
        prev_seq_num = -1;
    }

    if tail {
        // listening for incoming transactions
        loop {
            thread::sleep(time::Duration::from_millis(1000));
            let resp = get_account_transactions_response(
                &client,
                address,
                &network,
                prev_seq_num + 1,
                100,
            )
            .await?;
            let json_with_txns: serde_json::Value =
                serde_json::from_str(resp.text().await?.as_str())?;
            let txn_array = json_with_txns
                .as_array()
                .ok_or_else(|| anyhow!("Couldn't convert to array"))?
                .to_vec();

            // checking if there are transactions
            if txn_array.is_empty() {
                continue;
            }
            let last_txn_seq_num = parse_txn_for_seq_num(
                txn_array
                    .last()
                    .ok_or_else(|| anyhow!("Couldn't get last transaction"))?,
            )?;
            if last_txn_seq_num > prev_seq_num {
                write_out_txns(txn_array, &mut io::stdout(), raw)?;
            }
            prev_seq_num = last_txn_seq_num;
        }
    }
    Ok(())
}

fn write_out_txns<W: Write>(all_transactions: Vec<Value>, mut stdout: W, raw: bool) -> Result<()> {
    for txn in all_transactions.iter() {
        write_into(&mut stdout, txn, raw)?;
    }

    Ok(())
}

async fn get_account_transactions_response(
    client: &Client,
    address: AccountAddress,
    network: &Url,
    start: i64,
    limit: u64,
) -> Result<Response> {
    let path =
        network.join(format!("accounts/{}/transactions", address.to_hex_literal()).as_str())?;
    Ok(client
        .get(path.as_str())
        .query(&[("start", start.to_string().as_str())])
        .query(&[("limit", limit.to_string().as_str())])
        .send()
        .await?)
}

async fn get_account_sequence_number(
    client: &Client,
    network: &Url,
    address: AccountAddress,
) -> Result<u64> {
    let path =
        network.join(format!("accounts/{}/resources/", address.to_hex_literal()).as_str())?;
    let resp = client.get(path.as_str()).send().await?;
    check_accounts_resources_response_status_code(&resp.status())?;
    let json: Vec<Value> = serde_json::from_str(resp.text().await?.as_str())?;
    parse_json_for_account_seq_num(json)
}

fn check_accounts_resources_response_status_code(status_code: &StatusCode) -> Result<()> {
    match status_code == &StatusCode::from_u16(200)? {
        true => Ok(()),
        false => Err(anyhow!(
            "Failed to get account resources with provided address"
        )),
    }
}

fn parse_txn_for_seq_num(last_txn: &Value) -> Result<i64> {
    Ok(last_txn["sequence_number"]
        .to_string()
        .replace('"', "")
        .parse::<i64>()?)
}

fn parse_json_for_account_seq_num(json_objects: Vec<Value>) -> Result<u64> {
    let mut seq_number_string = "";
    for object in &json_objects {
        if object["type"] == DIEM_ACCOUNT_TYPE {
            seq_number_string = object["value"]["sequence_number"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid sequence number string"))?;
            break;
        };
    }
    let seq_number: u64 = seq_number_string.parse()?;
    Ok(seq_number)
}

fn write_into<W>(writer: &mut W, json: &serde_json::Value, raw: bool) -> io::Result<()>
where
    W: Write,
{
    match raw {
        true => writeln!(writer, "{}", json),
        false => writeln!(writer, "{:#}", json),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{json, Value};

    fn get_sample_txn() -> Value {
        json!([{
            "type":"user_transaction",
            "version":"268",
            "hash":"0x8be63c23e88f9d0290f060e33fe09e8a755e45f41ee0fc9447f3b5d97a8b88d1",
            "state_root_hash":"0x8d903f9df4092946f036164fa925c6716a172ee0140f2404371393e517b7058e",
            "event_root_hash":"0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000",
            "gas_used":"8",
            "success":true,
            "vm_status":"Executed successfully",
            "sender":"0x24163afcc6e33b0a9473852e18327fa9",
            "sequence_number":"2",
            "max_gas_amount":"1000000",
            "gas_unit_price":"0",
            "gas_currency_code":"XUS",
            "expiration_timestamp_secs":"1635800460",
            "payload":{}
        },{
            "type":"user_transaction",
            "version":"270",
            "hash":"0x42343251",
            "state_root_hash":"0x3434235",
            "event_root_hash":"0x3434235",
            "gas_used":"5",
            "success":true,
            "vm_status":"Executed successfully",
            "sender":"0x24163afcc6e33b0a9473852e18327fa9",
            "sequence_number":"3",
            "max_gas_amount":"1000000",
            "gas_unit_price":"0",
            "gas_currency_code":"XUS",
            "expiration_timestamp_secs":"1635800460",
            "payload":{}
        }])
    }

    #[test]
    fn test_parse_json_for_seq_num() {
        let value_obj = json!({
            "type":"0x1::DiemAccount::DiemAccount",
            "value": {
                "authentication_key": "0x88cae30f0fea7879708788df9e7c9b7524163afcc6e33b0a9473852e18327fa9",
                "key_rotation_capability":{
                    "vec":[{"account_address":"0x24163afcc6e33b0a9473852e18327fa9"}]
                },
                "received_events":{
                    "counter":"0",
                    "guid":{}
                },
                "sent_events":{},
                "sequence_number":"3",
                "withdraw_capability":{
                    "vec":[{"account_address":"0x24163afcc6e33b0a9473852e18327fa9"}]
                }
            }
        });

        let json_obj: Vec<Value> = vec![value_obj];
        let ret_seq_num = parse_json_for_account_seq_num(json_obj).unwrap();
        assert_eq!(ret_seq_num, 3);
    }

    #[test]
    fn test_write_into() {
        let mut stdout = Vec::new();
        let txn = get_sample_txn();
        write_into(&mut stdout, &txn[0], false).unwrap();
        assert_eq!(
            format!("{:#}\n", txn[0]),
            String::from_utf8(stdout).unwrap().as_str()
        );

        stdout = Vec::new();
        write_into(&mut stdout, &txn[0], true).unwrap();
        assert_eq!(
            format!("{}\n", txn[0]),
            String::from_utf8(stdout).unwrap().as_str()
        );
    }

    #[test]
    fn test_write_out_txns_stdout() {
        let all_txns = get_sample_txn();
        let txn_array = all_txns.as_array().unwrap();
        let mut stdout = Vec::new();
        write_out_txns(txn_array.to_vec(), &mut stdout, false).unwrap();
        assert_eq!(
            String::from_utf8(stdout).unwrap().as_str(),
            format!("{:#}\n{:#}\n", txn_array[0], txn_array[1])
        );

        stdout = Vec::new();
        write_out_txns(txn_array.to_vec(), &mut stdout, true).unwrap();
        assert_eq!(
            String::from_utf8(stdout).unwrap().as_str(),
            format!("{}\n{}\n", txn_array[0], txn_array[1])
        )
    }

    #[test]
    fn test_parse_seq_num() {
        let txn = get_sample_txn();
        let seq_num = parse_txn_for_seq_num(&txn[0]).unwrap();
        assert_eq!(seq_num, 2);
    }

    #[test]
    fn test_check_accounts_resources_response_status_code() {
        assert_eq!(
            check_accounts_resources_response_status_code(&StatusCode::from_u16(200).unwrap())
                .is_err(),
            false
        );
        assert_eq!(
            check_accounts_resources_response_status_code(&StatusCode::from_u16(404).unwrap())
                .is_err(),
            true
        );
    }
}

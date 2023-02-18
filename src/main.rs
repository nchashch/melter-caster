#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod client;

use crate::client::ZcashClient;
use anyhow::Result;
use bitcoin::Amount;
use chrono::prelude::*;
use chrono::Duration;
use num_traits::cast::FromPrimitive;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct Bill(usize);

impl Bill {
    fn new(bill: usize) -> Option<Self> {
        if bill > 63 {
            return None;
        }
        Some(Self(bill))
    }

    fn amount(&self) -> Amount {
        let amount = 1_u64 << self.0;
        Amount::from_sat(amount)
    }

    fn weekday(&self) -> Weekday {
        let day = self.0 % 7;
        Weekday::from_usize(day).unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let zcash = client::new_client()?;
    dbg!(zcash.getnewaddress(None).await);
    dbg!(zcash.z_getnewaddress().await);
    // let bills = cast(428178451755543, 6);
    // let mut schedule = generate_schedule(Utc::now(), &bills);
    // println!("Cast schedule:");
    // schedule.sort_by(|a, b| a.0.cmp(&b.0));
    // for (time, bills) in schedule {
    //     let amounts: Vec<Amount> = bills.iter().map(Bill::amount).collect();
    //     let total: u64 = amounts.iter().copied().map(Amount::to_sat).sum();
    //     println!("{}: {}", time, Amount::from_sat(total),);
    // }
    // loop {
    //     let next_cast_time = get_next_cast_time(Utc::now());
    //     let bills = get_bills_for_weekday(next_cast_time.weekday(), &bills);
    //     let time_until_cast = next_cast_time - Utc::now();
    //     let hours = time_until_cast.num_hours();
    //     let minutes = time_until_cast.num_minutes() - hours * 60;
    //     let amounts: Vec<Amount> = bills.iter().map(Bill::amount).collect();
    //     let total: u64 = amounts.iter().copied().map(Amount::to_sat).sum();
    //     println!(
    //         "{} will be cast in {} hours {} minutes",
    //         Amount::from_sat(total),
    //         hours,
    //         minutes
    //     );
    //     if time_until_cast < Duration::minutes(1) {
    //         // send zaddr -> taddr transactions
    //         println!("casting");
    //     }
    //     std::thread::sleep(std::time::Duration::from_secs(30));
    // }
    Ok(())
}

fn get_next_cast_time(now: DateTime<Utc>) -> DateTime<Utc> {
    let deadline = {
        let date = now.naive_utc().date();
        let naive_deadline = NaiveDateTime::new(date, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        DateTime::<Utc>::from_utc(naive_deadline, Utc)
    };
    match now < deadline {
        true => deadline,
        false => deadline + Duration::days(1),
    }
}

fn get_bills_for_weekday(weekday: Weekday, bills: &[Bill]) -> Vec<Bill> {
    bills
        .iter()
        .copied()
        .filter(|bill| bill.weekday() == weekday)
        .collect()
}

fn cast(amount: u64, max_bills: usize) -> Vec<Bill> {
    let mut bills = vec![];
    for i in (0..64).rev() {
        let bill = 1_u64 << i;
        if (amount & bill) > 0 {
            let bill = Bill::new(i).unwrap();
            bills.push(bill);
        }
        if bills.len() == max_bills {
            break;
        }
    }
    bills
}

fn get_weekday(bill: usize) -> Option<Weekday> {
    if bill > 63 {
        return None;
    }
    let day = bill % 7;
    Weekday::from_usize(day)
}

fn generate_schedule(now: DateTime<Utc>, bills: &[Bill]) -> Vec<(DateTime<Utc>, Vec<Bill>)> {
    let mut weekday_to_bills: HashMap<Weekday, Vec<Bill>> = HashMap::from([
        (Weekday::Mon, vec![]),
        (Weekday::Tue, vec![]),
        (Weekday::Wed, vec![]),
        (Weekday::Thu, vec![]),
        (Weekday::Fri, vec![]),
        (Weekday::Sat, vec![]),
        (Weekday::Sun, vec![]),
    ]);
    for bill in bills {
        let weekday = bill.weekday();
        let weekday_bills = weekday_to_bills.get_mut(&weekday).unwrap();
        weekday_bills.push(*bill);
    }
    let deadline = {
        let date = now.naive_utc().date();
        let naive_deadline = NaiveDateTime::new(date, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        DateTime::<Utc>::from_utc(naive_deadline, Utc)
    };
    let start_date = match now < deadline {
        true => deadline,
        false => deadline + Duration::days(1),
    };
    let mut schedule: Vec<(DateTime<Utc>, Vec<Bill>)> = vec![];
    let mut date = start_date;
    while !weekday_to_bills.is_empty() {
        let bills = weekday_to_bills[&date.weekday()].clone();
        if !bills.is_empty() {
            schedule.push((date, bills));
        }
        weekday_to_bills.remove(&date.weekday());
        date += Duration::days(1);
    }
    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn prop(unix_secs: i32, amounts: Vec<u64>) -> bool {
            let bills: Vec<Bill> = amounts.iter().flat_map(|amount| cast(*amount, 64)).collect();
            let now = NaiveDateTime::from_timestamp_opt(unix_secs as i64, 0).unwrap();
            let now = DateTime::<Utc>::from_utc(now, Utc);
            let schedule = generate_schedule(now, &bills);
            dbg!(now, schedule);
            true
        }
    }
}

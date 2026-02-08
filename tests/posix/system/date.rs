//! POSIX.1-2017 compliance tests for date
//!
//! Reference: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/date.html

use crate::posix::helpers::*;

/// POSIX: "The date utility shall write the date and time to standard output"
#[test]
fn posix_date_basic() {
    let result = run(&["date"]);
    assert_success(&result);
    assert!(!result.1.is_empty());
}

/// POSIX: date +format custom format
#[test]
fn posix_date_format() {
    let result = run(&["date", "+%Y"]);
    assert_success(&result);
    // Should be a 4-digit year
    let year = result.1.trim();
    assert!(year.len() == 4);
    assert!(year.chars().all(|c| c.is_ascii_digit()));
}

/// POSIX: date +%Y year
#[test]
fn posix_date_year() {
    let result = run(&["date", "+%Y"]);
    assert_success(&result);
    let year: i32 = result.1.trim().parse().unwrap();
    assert!(year >= 2020 && year <= 2100);
}

/// POSIX: date +%m month
#[test]
fn posix_date_month() {
    let result = run(&["date", "+%m"]);
    assert_success(&result);
    let month: i32 = result.1.trim().parse().unwrap();
    assert!(month >= 1 && month <= 12);
}

/// POSIX: date +%d day
#[test]
fn posix_date_day() {
    let result = run(&["date", "+%d"]);
    assert_success(&result);
    let day: i32 = result.1.trim().parse().unwrap();
    assert!(day >= 1 && day <= 31);
}

/// POSIX: date +%H hour
#[test]
fn posix_date_hour() {
    let result = run(&["date", "+%H"]);
    assert_success(&result);
    let hour: i32 = result.1.trim().parse().unwrap();
    assert!(hour >= 0 && hour <= 23);
}

/// POSIX: date +%M minute
#[test]
fn posix_date_minute() {
    let result = run(&["date", "+%M"]);
    assert_success(&result);
    let minute: i32 = result.1.trim().parse().unwrap();
    assert!(minute >= 0 && minute <= 59);
}

/// POSIX: date +%S second
#[test]
fn posix_date_second() {
    let result = run(&["date", "+%S"]);
    assert_success(&result);
    let second: i32 = result.1.trim().parse().unwrap();
    assert!(second >= 0 && second <= 60); // 60 for leap seconds
}

/// POSIX: date -u UTC time
#[test]
fn posix_date_utc() {
    let result = run(&["date", "-u"]);
    assert_success(&result);
    // Should contain UTC or GMT indicator
}

/// POSIX: Exit status 0 on success
#[test]
fn posix_date_exit_success() {
    let result = run(&["date"]);
    assert_eq!(result.0, 0);
}

/// POSIX: date +%j day of year
#[test]
fn posix_date_day_of_year() {
    let result = run(&["date", "+%j"]);
    assert_success(&result);
    let day: i32 = result.1.trim().parse().unwrap();
    assert!(day >= 1 && day <= 366);
}

/// POSIX: date +%w day of week (0=Sunday)
#[test]
fn posix_date_day_of_week() {
    let result = run(&["date", "+%w"]);
    assert_success(&result);
    let dow: i32 = result.1.trim().parse().unwrap();
    assert!(dow >= 0 && dow <= 6);
}

/// POSIX: date combined format
#[test]
fn posix_date_combined_format() {
    let result = run(&["date", "+%Y-%m-%d %H:%M:%S"]);
    assert_success(&result);
    // Should be in format YYYY-MM-DD HH:MM:SS
    let output = result.1.trim();
    assert!(output.len() == 19);
}

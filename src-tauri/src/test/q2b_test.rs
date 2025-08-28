use crate::utils::q2b::*;
use mongodb::bson::{doc, Bson, DateTime, Decimal128, Document,to_document, oid::ObjectId};
use regex::Regex;
use serde_json::Value;

#[test]
fn test_parse_special_wrapper() {
	// 测试 ISODate 格式
	assert_eq!(
		parse_special_wrapper("ISODate(\"2025-08-28T00:00:00Z\")", "ISODate"),
		Some("\"2025-08-28T00:00:00Z\"")
	);

	// 测试 ObjectId 格式
	assert_eq!(
		parse_special_wrapper("ObjectId(\"507f1f77bcf86cd799439011\")", "ObjectId"),
		Some("\"507f1f77bcf86cd799439011\"")
	);

	// 测试 NumberDecimal 格式
	assert_eq!(
		parse_special_wrapper("NumberDecimal(\"123.456\")", "NumberDecimal"),
		Some("\"123.456\"")
	);

	// 测试无效格式
	assert_eq!(parse_special_wrapper("Invalid(\"123\")", "ISODate"), None);
}

#[test]
fn test_preprocess_query() {
	// 测试正则预处理
	let query = "ISODate(\"2025-08-28T00:00:00Z\")";
	let processed = preprocess_query(query);
	assert_eq!(
		processed,
		"\"ISODate(2025-08-28T00:00:00Z)\""
	);
}

#[test]
fn test_convert_value() {
	// 测试 ISODate 转换
	let date_str = "ISODate(2025-08-28T00:00:00Z)";
	let bson_str = Bson::String(date_str.to_string());
	let converted = convert_value(bson_str);
	assert!(matches!(converted, Bson::DateTime(_)));

	// 测试 ObjectId 转换
	let oid_str = "ObjectId(507f1f77bcf86cd799439011)";
	let bson_str = Bson::String(oid_str.to_string());
	let converted = convert_value(bson_str);
	assert!(matches!(converted, Bson::ObjectId(_)));

	// 测试 NumberDecimal 转换
	let nd_str = "NumberDecimal(123.456)";
	let bson_str = Bson::String(nd_str.to_string());
	let converted = convert_value(bson_str);
	assert!(matches!(converted, Bson::Decimal128(_)));

	// 测试无效字符串
	let invalid_str = "Invalid(123)";
	let bson_str = Bson::String(invalid_str.to_string());
	let converted = convert_value(bson_str);
	assert!(matches!(converted, Bson::String(_)));
}

#[test]
fn test_convert_document() {
	// 测试嵌套文档转换
	let query = r#"
		{
		date: ISODate("2025-08-28T00:00:00Z"),
		oid: ObjectId("507f1f77bcf86cd799439011"),
		decimal: NumberDecimal("123.456"),
		nested: {
			date: ISODate("2025-08-28T00:00:00Z")
			}
		}
	"#;
	let json: Value = json5::from_str(&preprocess_query(query)).unwrap();
	let doc = to_document(&json).unwrap();
	let converted = convert_document(doc);
	assert!(matches!(converted.get("date"), Some(Bson::DateTime(_))));
	assert!(matches!(converted.get("oid"), Some(Bson::ObjectId(_))));
	assert!(matches!(converted.get("decimal"), Some(Bson::Decimal128(_))));
	assert!(matches!(
		converted.get("nested").and_then(|v| v.as_document()),
		Some(_)
	));
}
use log::error;
use mongodb::bson::{oid::ObjectId, Bson, DateTime, Decimal128, Document};
use regex::Regex;
use std::sync::LazyLock;

pub static MONGO_SPECIAL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r##"(?x)
        (ISODate|ObjectId|NumberDecimal)  # 方法名
        \(                                                              # 左括号
        ['"]                                                            # 引号开始
        (?P<content>[^'"]+)                                             # 内容
        ['"]                                                            # 引号结束
        \)                                                              # 右括号
        "##,
    )
    .unwrap()
});

// 转换文档中的所有值
pub fn convert_document(doc: Document) -> Document {
    doc.into_iter()
        .map(|(k, v)| (k, convert_value(v)))
        .collect()
}

// 转换单个值，处理ISODate字符串
pub fn convert_value(value: Bson) -> Bson {
    match value {
        // 如果是嵌套文档，递归处理
        Bson::Document(doc) => Bson::Document(convert_document(doc)),
        // 如果是数组，逐个处理元素
        Bson::Array(arr) => Bson::Array(arr.into_iter().map(convert_value).collect()),
        // 如果是字符串，检查是否为支持格式
        Bson::String(s) => {
            if let Some(date_str) = parse_special_wrapper(&s, "ISODate") {
                match DateTime::parse_rfc3339_str(date_str) {
                    Ok(dt) => Bson::DateTime(dt),
                    Err(e) => {
                        error!("Failed to parse date '{}': {}", date_str, e);
                        Bson::String(s)
                    }
                }
            } else if let Some(oid_str) = parse_special_wrapper(&s, "ObjectId") {
                match ObjectId::parse_str(oid_str) {
                    Ok(oid) => Bson::ObjectId(oid),
                    Err(e) => {
                        error!("Failed to parse ObjectId '{}': {}", oid_str, e);
                        Bson::String(s)
                    }
                }
            } else if let Some(nd_str) = parse_special_wrapper(&s, "NumberDecimal") {
                match nd_str.parse::<Decimal128>() {
                    Ok(nd) => Bson::Decimal128(nd),
                    Err(e) => {
                        error!("Failed to parse Decimal128 '{}': {}", nd_str, e);
                        Bson::String(s)
                    }
                }
            } else {
                Bson::String(s)
            }
        }
        // 其他类型保持不变 or _ => value,
        other => other,
    }
}

// 解析预处理后的格式
pub fn parse_special_wrapper<'a>(
	s: &'a str, 
	method_name: &str
) -> Option<&'a str> {
    let prefix = format!("{}(", method_name);
    if s.starts_with(&prefix) && s.ends_with(')') {
        // 提取方法名( 和 ) 之间的内容
        let content = &s[prefix.len()..s.len() - 1];
        Some(content)
    } else {
        None
    }
}

// 使用正则表达式进行预处理
pub fn preprocess_query(query: &str) -> String {
    MONGO_SPECIAL_REGEX
        .replace_all(query, r#""$1($content)""#)
        .to_string()
}
